/*
 * endpoints/misc.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::info;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use std::path::PathBuf;
use wikidot_normalize::normalize;

async fn postgres_check(ctx: &ServiceContext<'_>) -> Result<()> {
    ctx.transaction()
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            str!("SELECT 1"),
        ))
        .await?;

    debug!("Successfully pinged Postgres");
    Ok(())
}

async fn redis_check(ctx: &ServiceContext<'_>) -> Result<()> {
    let mut redis = ctx.redis_connect().await?;

    redis
        .send_packed_command(redis::Cmd::new().arg("PING"))
        .await?;

    debug!("Successfully pinged Redis");
    Ok(())
}

pub async fn ping(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<&'static str> {
    // Ensure the database and cache are connected, and only then return.
    info!("Ping request");
    try_join!(postgres_check(ctx), redis_check(ctx))?;
    Ok("Pong!")
}

/// Method which always returns an error.
/// For testing.
pub async fn yield_error(
    _ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<()> {
    info!("Returning DEEPWELL error for testing");
    Err(ServiceError::BadRequest)
}

pub async fn version(
    _ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<&'static str> {
    info!("Getting DEEPWELL version");
    Ok(info::VERSION.as_str())
}

pub async fn full_version(
    _ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<&'static str> {
    info!("Getting DEEPWELL version (full)");
    Ok(info::FULL_VERSION.as_str())
}

pub async fn hostname(
    _ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<&'static str> {
    info!("Getting DEEPWELL hostname");
    Ok(info::HOSTNAME.as_str())
}

pub async fn config_dump(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<String> {
    info!("Dumping raw DEEPWELL configuration for debugging");
    Ok(ctx.config().raw_toml.to_string())
}

pub async fn config_path(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<PathBuf> {
    info!("Dumping DEEPWELL configuration path for debugging");
    Ok(ctx.config().raw_toml_path.to_path_buf())
}

pub async fn normalize_method(
    _ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<String> {
    let mut value: String = params.one()?;
    info!("Running normalize on string: {value:?}");
    normalize(&mut value);
    Ok(value)
}
