/*
 * services/job/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use cron::Schedule;

#[derive(Debug, Clone)]
pub struct Job {
    /// The body of the job, what action to perform.
    pub action: JobAction,

    /// On what regular cadence to perform this job.
    ///
    /// If `None`, then the job is performed immediately
    /// and then discarded.
    pub schedule: Option<Schedule>,
}

impl Job {
    #[inline]
    pub fn cron(action: JobAction, schedule: Schedule) -> Self {
        Job {
            action,
            schedule: Some(schedule),
        }
    }

    #[inline]
    pub fn now(action: JobAction) -> Self {
        Job {
            action,
            schedule: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum JobAction {
    RerenderPage { site_id: i64, page_id: i64 },
}
