/*
 * services/page_query/service.rs
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

use std::borrow::Cow;

use super::prelude::*;
use crate::models::{
    page::{self, Entity as Page},
    page_category::{self, Entity as PageCategory},
    page_connection::{self, Entity as PageConnection},
    page_parent::{self, Entity as PageParent},
    page_revision,
};
use crate::web::SerialReference;
use crate::services::PageService;
use sea_query::{all, Expr, Query};

#[derive(Debug)]
pub struct PageQueryService;

impl PageQueryService {
    pub async fn construct_query<'a>(
        ctx: &ServiceContext<'_>,
        PageQuery {
            current_page_id,
            queried_site_id,
            page_type,
            categories,
            tags,
            page_parent,
            contains_outgoing_links,
            creation_date,
            update_date,
            author,
            rating,
            votes,
            offset,
            range,
            name,
            slug,
            data_form_fields,
            order,
            pagination,
            variables,
        }: PageQuery<'_>,
    ) -> Result<bool> /* Result<PageQueryOutput<'a>> */ {
        let txn = ctx.transaction();

        let mut condition = Condition::all();

        // Queried Site ID
        //
        // The site to query. If not specified, will query current site.
        condition = condition.add(page::Column::SiteId.eq(queried_site_id));

        // Page Type Filtering
        //
        // If a specific page type is requested, check if the slug does or does not begin
        // with an underscore (which indicates if a page is hidden).
        match page_type {
            PageTypeSelector::Normal => {
                condition = condition.add(page::Column::Slug.not_like("_%"))
            } // TODO: https://github.com/SeaQL/sea-orm/issues/1221

            PageTypeSelector::Hidden => {
                condition = condition.add(page::Column::Slug.starts_with("_"))
            }

            PageTypeSelector::All => {}
        };

        // Category Filtering
        //
        // Adds a condition based on the catgeories that are included/excluded from the query.
        condition = match categories.included_categories {
            // If all categories are selected (using an asterisk or by only specifying excluded categories),
            // then filter only by site_id and exclude the specified excluded categories.
            IncludedCategories::All => condition.add(
                page::Column::PageCategoryId.in_subquery(
                    Query::select()
                        .column(page_category::Column::CategoryId)
                        .from(PageCategory)
                        .and_where(page_category::Column::SiteId.eq(queried_site_id))
                        .and_where(page_category::Column::Slug.is_not_in(
                            categories.excluded_categories.iter().map(|c| c.as_ref()),
                        ))
                        .to_owned(),
                ),
            ),

            // If a specific list of categories is provided, filter by site_id, inclusion in the specified included categories,
            // and exclude the specified excluded categories.
            //
            // NOTE: Exclusion can only have an effect in this query if it is *also* included. Although by definition
            // this is the same as not including the category in the included categories to begin with, it is still
            // accounted for to preserve backwards-compatibility with poorly-constructed listpages modules.
            IncludedCategories::List(included_categories) => condition.add(
                page::Column::PageCategoryId.in_subquery(
                    Query::select()
                        .column(page_category::Column::CategoryId)
                        .from(PageCategory)
                        .and_where(page_category::Column::SiteId.eq(queried_site_id))
                        .and_where(
                            page_category::Column::Slug
                                .is_in(included_categories.iter().map(|c| c.as_ref())),
                        )
                        .and_where(page_category::Column::Slug.is_not_in(
                            categories.excluded_categories.iter().map(|c| c.as_ref()),
                        ))
                        .to_owned(),
                ),
            ),
        };

        // Page Parent Selector
        //
        // Defines the relationship of the pages being queried with their parent pages.
        condition = match page_parent {
            // Pages with no parents, meaning they are not members of the page_parent table as children pages.
            PageParentSelector::NoParent => condition.add(
                page::Column::PageId.not_in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .to_owned(),
                ),
            ),

            // Pages with one of the same parents as the current page.
            PageParentSelector::SameParents(parents) => condition.add(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(page_parent::Column::ParentPageId.is_in(parents))
                        .to_owned(),
                ),
            ),

            // Pages with none of the current page's parents.
            PageParentSelector::DifferentParents(parents) => condition.add(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(page_parent::Column::ParentPageId.is_not_in(parents))
                        .to_owned(),
                ),
            ),

            // Children pages of the current page.
            PageParentSelector::ChildOf => condition.add(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(page_parent::Column::ParentPageId.eq(current_page_id))
                        .to_owned(),
                ),
            ),

            // Pages with any of the specified parents.
            // TODO: Possibly allow either *any* or *all* of specified parents rather than only any in the future.
            PageParentSelector::HasParents(parents) => {
                let parent_ids = PageService::get_ids(
                    ctx,
                    queried_site_id,
                    &parents
                        .iter()
                        .map(|r| r.as_reference())
                        .collect::<Vec<Reference>>(),
                )
                .await?
                .into_iter()
                .filter_map(|id| id);

                condition.add(
                    page::Column::PageId.in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .and_where(
                                page_parent::Column::ParentPageId.is_in(parent_ids),
                            )
                            .to_owned(),
                    ),
                )
            }
        };

        // Slug Condition
        //
        // Whether the page's slug is equal to the one provided.
        condition = condition.add(page::Column::Slug.eq(slug.as_ref()));

        // Contains Link Selector
        //
        // Selects pages that have an outgoing link (`FromPageId`) to a specified page (`ToPageId`).
        condition = condition.add(
            page::Column::PageId.in_subquery(
                Query::select()
                    .column(page_connection::Column::FromPageId)
                    .from(PageConnection)
                    .and_where(
                        // Get Page IDs from Reference structs. Uses result to only select pages that
                        // have an outgoing link to those pages.
                        page_connection::Column::ToPageId.is_in(
                            PageService::get_ids(
                                ctx,
                                queried_site_id,
                                &contains_outgoing_links
                                    .iter()
                                    .map(|r| r.as_reference())
                                    .collect::<Vec<Reference>>(),
                            )
                            .await?
                            .into_iter()
                            .filter_map(|id| id),
                        )
                    ).to_owned(),
            ),
        );

        // Converts tag fields of Vec<Cow<str>> to Vec<String> for use in tag filtering.
        let (any_tags, all_tags, no_tags) = (
            convert_cow_to_string_vec(tags.any_present),
            convert_cow_to_string_vec(tags.all_present),
            convert_cow_to_string_vec(tags.none_present),
        );

        // Execute the query itself.
        let result = Page::find()
            .filter(condition)
            // Tag Filtering
            //
            // Performs an inner join connecting the `page` table with the `page_revision` table.
            // Only the latest revision per page is joined by equating latest revision IDs and the Page IDs.
            .join(
                JoinType::LeftJoin,
                page::Relation::PageRevision
                    .def()
                    .on_condition(move |left, right| {
                        all![
                            // Only select the page's latest revision via revision ID.
                            Expr::tbl(left.clone(), page::Column::LatestRevisionId)
                                .equals(right.clone(), page_revision::Column::RevisionId),
                            // Ensure pages match by page ID.
                            // TODO: Check if this condition is superfluous.
                            Expr::tbl(left, page::Column::PageId)
                                .equals(right.clone(), page_revision::Column::PageId),
                            // Use Postgres "array overlap" operator to select pages that have any of the following inputted tags.
                            // Uses `cust_with_values` since the operator doesn't have a Seaquery equivalent.
                            Expr::cust_with_values(
                                r#""page_revision"."tags" && $1"#,
                                [any_tags.clone()]
                            ),
                            // Select all pages that have *all* of the following inputted tags.
                            Expr::tbl(right, page_revision::Column::Tags)
                                .contains(Expr::val(all_tags.clone())),
                            // Use NOT of Postgres "array overlap" operator to select pages that have none of the following inputted tags.
                            // Uses `cust_with_values` since the operator doesn't have a Seaquery equivalent.
                            Expr::cust_with_values(
                                r#"NOT "page_revision"."tags" && $1"#,
                                [no_tags.clone()]
                            )
                        ]
                    }),
            )
            .all(txn)
            .await?;

        /* TODO: creation_date, update_date, rating, votes, offset, range, name, data_form_fields,
        order, pagination, variables */

        return Ok(true);
    }
}

fn convert_cow_to_string_vec(vector: Vec<Cow<str>>) -> Vec<String> {
    vector.iter().map(|c| c.as_ref().to_string()).collect()
}
