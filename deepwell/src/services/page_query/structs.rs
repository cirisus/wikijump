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
use regex::Regex;

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
    HasParents(&'a [&'a str]),
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
pub struct RatingSelector {
    pub rating: f64,
    pub comparison: ComparisonOperation,
}

#[derive(Debug)]
pub enum RangeSelector {
    Current,
    Before(u32),
    After(u32),
    Others,
}

#[derive(Debug)]
pub struct DataFormSelector<'a> {
    pub field: &'a str,
    pub value: &'a str,
}

#[derive(Debug)]
pub enum OrderProperties {
    Name,
    Fullname,
    Title,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
    Size,
    Rating,
    Votes,
    Revisions,
    Comments,
    Random,
    DataFormFieldName,
}

#[derive(Debug)]
pub struct OrderBySelector {
    pub property: OrderProperties,
    pub ascending: bool,
}

#[derive(Debug)]
pub struct PaginationSelector {
    pub limit: u64,
    pub per_page: u8,
    pub reversed: bool,
}

#[derive(Debug)]
pub enum PageQueryVariables<'a> {
    CreatedAt,
    CreatedBy,
    CreatedByUnix,
    CreayedByID,
    CreatedByLinked,
    UpdatedAt,
    UpdatedBy,
    UpdatedByUnix,
    UpdatedByID,
    UpdatedByLinked,
    CommentedAt,
    CommentedBy,
    CommentedByUnix,
    CommentedByID,
    CommentedByLinked,
    Name,
    Category,
    Fullname,
    Title,
    TitleLinked,
    ParentNamed,
    ParentCategory,
    ParentFullname,
    ParentTitle,
    ParentTitleLinked,
    Link,
    Content,
    ContentN(u64),
    Preview,
    PreviewN(u64),
    Summary,
    FirstParagraph,
    Tags,
    TagsLinked,
    TagsLinkedURL(&'a str),
    HiddenTags,
    HiddenTagsLinked,
    HiddenTagsLinkedURL(&'a str),
    FormData(&'a str),
    FormRaw(&'a str),
    FormLabel(&'a str),
    FormHint(&'a str),
    Children,
    Comments,
    Size,
    Rating,
    RatingVotes,
    RatingPercent,
    Revisions,
    Index,
    Total,
    Limit,
    TotalOrLimit,
    SiteTitle,
    SiteName,
    SiteDomain,
}

#[derive(Debug)]
pub struct CreatePageQuery<'a> {
    pub current_page_id: u64,
    pub page_type: PageTypeSelector,
    pub categories: CategoriesSelector<'a>,
    pub tags: &'a [TagCondition<'a>],
    pub page_parent: PageParentSelector<'a>,
    pub contains_outgoing_links: &'a [&'a str],
    pub creation_date: DateSelector,
    pub update_date: DateSelector,
    pub author: &'a str,
    pub rating: &'a [RatingSelector], // 5-star rating selector
    pub votes: &'a [RatingSelector],  // upvote/downvote rating selector
    pub offset: u32,
    pub range: RangeSelector,
    pub name: Regex,
    pub fullname: &'a str,
    pub data_form_fields: &'a [DataFormSelector<'a>],
    pub order: OrderBySelector,
    pub pagination: PaginationSelector,
    pub variables: &'a [PageQueryVariables<'a>],
}
