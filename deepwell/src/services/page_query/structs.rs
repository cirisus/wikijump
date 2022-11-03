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

use std::borrow::Cow;

/// The type of page based on visibility to select in a page query.
/// A page is hidden if its URL is prefixed by an underscore; otherwise, it is visible.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PageTypeSelector {
    Normal,
    Hidden,
    All,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub enum IncludedCategories<'a> {
    All,
    List(Cow<'a, [Cow<'a, str>]>),
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct CategoriesSelector<'a> {
    pub included_categories: IncludedCategories<'a>,
    pub excluded_categories: Cow<'a, [Cow<'a, str>]>,
}

/// The tag conditions for the page query.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub enum TagCondition<'a> {
    /// Represents an OR operator for the tag; page may contain any of these tags.
    AnyPresent(Cow<'a, str>),
    /// Represents the AND operator for the tag; page must contain all of these tags.
    AllPresent(Cow<'a, str>),
    /// Represents the NOT operator for the tag; page must *not* contain any of these tags.
    AllAbsent(Cow<'a, str>),
}

/// The relationship of the pages being queried to their parent/child pages.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Hash)]
pub enum PageParentSelector<'a> {
    /// Pages which do not have a parent page.
    NoParent,
    /// Pages which share the same parent page as the page making the query.
    SameParents,
    /// Pages which do *not* share the same parent page as the page making the query.
    DifferentParents,
    /// Pages which are children page of the page making the query.
    ChildOf,
    /// Pages which have specified parent pages.
    HasParents(Cow<'a, [Cow<'a, str>]>),
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
pub enum DateTimeResolution {
    Second,
    Minute,
    Hour,
    Day,
    Month,
    Year,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ComparisonOperation {
    GreaterThan,
    LessThan,
    GreaterOrEqualThan,
    LessOrEqualThan,
    Equal,
    NotEqual,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum DateSelector {
    /// A time span represented by a timestamp, the "resolution" of the time, and a comparison operator.
    Span {
        timestamp: DateTimeWithTimeZone,
        resolution: DateTimeResolution,
        comparison: ComparisonOperation,
    },

    /// A time span represented by a timestamp, from present to the time specified.
    FromPresent { start_time: DateTimeWithTimeZone },
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
pub struct RatingSelector {
    pub rating: f64,
    pub comparison: ComparisonOperation,
}

/// Range of pages to display, relative to the current page.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RangeSelector {
    /// Display only the current page.
    Current,
    /// Display pages before the current page in queried results.
    Before,
    /// Display pages after the current page in queried results.
    After,
    /// Display all pages besides the current page.
    Others,
}

/// Selects all pages that have a data form with matching field-value pairs.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct DataFormSelector<'a> {
    pub field: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
pub enum OrderProperties {
    Name,
    Slug,
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

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct OrderBySelector {
    pub property: OrderProperties,
    pub ascending: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PaginationSelector {
    pub limit: u64,
    pub per_page: u8,
    pub reversed: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub enum PageQueryVariables<'a> {
    CreatedAt,
    CreatedBy,
    CreatedBySlug,
    CreatedById,
    CreatedByLinked,
    UpdatedAt,
    UpdatedBy,
    UpdatedBySlug,
    UpdatedById,
    UpdatedByLinked,
    CommentedAt,
    CommentedBy,
    CommentedBySlug,
    CommentedById,
    CommentedByLinked,
    Name,
    Category,
    Slug,
    Title,
    TitleLinked,
    ParentNamed,
    ParentCategory,
    ParentSlug,
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
    TagsLinkedURL(Cow<'a, str>),
    HiddenTags,
    HiddenTagsLinked,
    HiddenTagsLinkedURL(Cow<'a, str>),
    FormData(Cow<'a, str>),
    FormRaw(Cow<'a, str>),
    FormLabel(Cow<'a, str>),
    FormHint(Cow<'a, str>),
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

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatePageQuery<'a> {
    pub current_page_id: u64,
    pub page_type: PageTypeSelector,
    pub categories: CategoriesSelector<'a>,
    pub tags: Vec<TagCondition<'a>>,
    pub page_parent: PageParentSelector<'a>,
    pub contains_outgoing_links: Vec<Cow<'a, str>>,
    pub creation_date: DateSelector,
    pub update_date: DateSelector,
    pub author: Cow<'a, str>,
    pub rating: Vec<RatingSelector>, // 5-star rating selector
    pub votes: Vec<RatingSelector>,  // upvote/downvote rating selector
    pub offset: u32,
    pub range: RangeSelector,
    pub name: Cow<'a, str>,
    pub slug: Cow<'a, str>,
    pub data_form_fields: Vec<DataFormSelector<'a>>,
    pub order: OrderBySelector,
    pub pagination: PaginationSelector,
    pub variables: Vec<PageQueryVariables<'a>>,
}
