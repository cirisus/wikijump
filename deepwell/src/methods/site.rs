/*
 * methods/site.rs
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
use crate::{
    models::site::Model as SiteModel,
    services::site::{CreateSite, UpdateSite},
};

pub async fn site_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from(&req)?;
    tide::log::info!("Creating site {:?}", reference);

    let input: CreateSite = req.body_json().await?;

    let output = SiteService::create(&ctx, input).await.to_api()?;
    txn.commit().await?;

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Created).body(body).into();
    Ok(response)
}

pub async fn site_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from(&req)?;
    tide::log::info!("Checking existence of site {:?}", reference);

    let exists = SiteService::exists(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    exists_status(exists)
}

pub async fn site_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from(&req)?;
    tide::log::info!("Getting site {:?}", reference);

    let site = SiteService::get(&ctx, reference).await.to_api()?;
    build_site_response(&site, StatusCode::Ok)
}

pub async fn site_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: UpdateSite = req.body_json().await?;
    let reference = Reference::try_from(&req)?;
    tide::log::info!("Updating site {:?}", reference);

    SiteService::update(&ctx, reference, input).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

fn build_site_response(site: &SiteModel, status: StatusCode) -> ApiResponse {
    let body = Body::from_json(site)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
