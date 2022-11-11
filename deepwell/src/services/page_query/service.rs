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
use sea_query::{Query, Expr};
use crate::models::page::{self, Entity as Page, Model as PageModel};

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

        let mut query = Query::select().from(Page);

        // If a specific page type is requested, check if the slug does or does not begin
        // with an underscore (which indicates if a page is hidden).
        match page_type {
            PageTypeSelector::Normal => query.and_where(Expr::col(page::Column::Slug).not_like("_%")),
            PageTypeSelector::Hidden => query.and_where(Expr::col(page::Column::Slug).like("_%")),
            PageTypeSelector::All => {},
        }

        /* TODO: categories, tags, page_parent, contains_outgoing_links,
        creation_date, update_date, rating, votes */

        // Offset by requested amount.
        query.offset(offset.into());

        /* TODO:  range, name, slug, data_form_fields, order, pagination, variables */
    }
}
