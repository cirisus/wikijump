/*
 * services/page_query/struct.rs
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

use crate::utils::DateTimeWithTimeZone;

use super::prelude::*;

#[derive(Debug)]
pub enum PageTypeSelector {
    Normal,
    Hidden,
    All,
}

#[derive(Debug)]
pub enum IncludedCategories<'a> {
    All,
    List(&'a [&'a str]),
}

#[derive(Debug)]
pub struct CategoriesSelector<'a> {
    pub included_categories: IncludedCategories<'a>,
    pub excluded_categories: &'a [&'a str],
}

#[derive(Debug)]
pub enum TagCondition<'a> {
    AnyPresent(&'a str),
    AllPresent(&'a str),
    AllAbsent(&'a str),
}

#[derive(Debug)]
pub enum PageParentSelector<'a> {
    NoParent,
    SameParents,
    DifferentParents,
    ChildOf,
    HasParents(&'a [&'a str])
}

#[derive(Debug)]
pub enum DateTimeResolution {
    Second,
    Minute,
    Hour,
    Day,
    Month,
    Year,
}

#[derive(Debug)]
pub enum ComparisonOperation {
    GreaterThan,
    LessThan,
    GreaterOrEqualThan,
    LessOrEqualThan,
    Equal,
    NotEqual,
}

#[derive(Debug)]
pub enum DateSelector {
    Span {
        timestamp: DateTimeWithTimeZone,
        resolution: DateTimeResolution,
        comparison: ComparisonOperation,
    },

    FromPresent {
        start_time: DateTimeWithTimeZone,
    },
}

#[derive(Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatePageQuery<'a> {
    pub current_page_id: u64,
    pub page_type: PageTypeSelector,
    pub categories: CategoriesSelector<'a>,
    pub tags: &'a [TagCondition<'a>],
    pub page_parent: PageParentSelector<'a>,
    pub contains_outgoing_links: &'a [&'a str],
    pub date: DateSelector,
}