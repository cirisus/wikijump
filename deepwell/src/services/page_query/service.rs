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

use super::prelude::*;
use crate::models::page::{self, Entity as Page, Model as PageModel};
use crate::models::page_category::{self, Entity as PageCategory};
use sea_query::Query;

#[derive(Debug)]
pub struct PageQueryService;

impl PageQueryService {
    pub async fn construct_query<'a>(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        PageQuery {
            current_page_id,
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
    ) -> Result<PageQueryOutput<'a>> {
        let txn = ctx.transaction();

        let mut condition = Condition::all();

        // If a specific page type is requested, check if the slug does or does not begin
        // with an underscore (which indicates if a page is hidden).
        match page_type {
            PageTypeSelector::Normal => {
                condition = condition.add(page::Column::Slug.starts_with("_"))
            }
            PageTypeSelector::Hidden => {
                condition = condition.add(page::Column::Slug.not_like("_%"))
            } // TODO: https://github.com/SeaQL/sea-orm/issues/1221
            PageTypeSelector::All => {}
        };

        // Adds a condition based on the catgeories that are included/excluded from the query.
        // Subqueries are necessary due to category information being stored in a separate table.
        condition =
            match categories.included_categories {
                // If all categories are selected (using an asterisk or by only specifying excluded categories),
                // then filter only by site_id and exclude the excluded categories.
                IncludedCategories::All => condition.add(
                    page::Column::PageCategoryId.in_subquery(
                        Query::select()
                            .column(page_category::Column::CategoryId)
                            .and_where(page_category::Column::SiteId.eq(site_id))
                            .and_where(
                                page_category::Column::Slug.is_not_in(
                                    categories
                                        .excluded_categories
                                        .into_iter()
                                        .map(|c| c.as_ref()),
                                ),
                            )
                            .to_owned(),
                    ),
                ),
                // If a specific list of categories is provided, filter by site_id, inclusion in the included categories,
                // and exclude the excluded categories.
                // NOTE: Exclusion can only have an effect in this query if it is *also* included. Although by definition
                // this is the same as not including the category in the included categories to begin with, it is still
                // accounted for to preserve backwards-compatibility with poorly-constructed listpages modules.
                IncludedCategories::List(included_categories) => condition.add(
                    page::Column::PageCategoryId.in_subquery(
                        Query::select()
                            .column(page_category::Column::CategoryId)
                            .and_where(page_category::Column::SiteId.eq(site_id))
                            .and_where(page_category::Column::Slug.is_in(
                                included_categories.into_iter().map(|c| c.as_ref()),
                            ))
                            .and_where(
                                page_category::Column::Slug.is_not_in(
                                    categories
                                        .excluded_categories
                                        .into_iter()
                                        .map(|c| c.as_ref()),
                                ),
                            )
                            .to_owned(),
                    ),
                ),
            }

        /* TODO: tags, page_parent, contains_outgoing_links, creation_date, update_date, rating, votes, offset,
        range, name, slug, data_form_fields, order, pagination, variables */
    }
}
