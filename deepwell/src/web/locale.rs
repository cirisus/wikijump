/*
 * web/file_details.rs
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

use crate::services::{Error, Result};
use unic_langid::LanguageIdentifier;

pub fn validate_locale(locale_str: &str) -> Result<LanguageIdentifier> {
    LanguageIdentifier::from_bytes(locale_str.as_bytes()).map_err(|error| {
        tide::log::warn!("Invalid locale '{}' passed: {:?}", locale_str, error);
        Error::BadRequest
    })
}
