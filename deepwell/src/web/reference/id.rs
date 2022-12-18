/*
 * web/reference/id.rs
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

use crate::api::ApiRequest;
use std::{borrow::Cow, convert::TryFrom};
use tide::{Error, StatusCode};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Reference<'a> {
    Id(i64),
    Slug(&'a str),
}

/// This is an enum for serializing and deserializing `Reference`.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum SerialReference<'a> {
    Id(i64),
    Slug(Cow<'a, str>),
}

impl<'a> Reference<'a> {
    pub fn try_from_fields_key(
        req: &'a ApiRequest,
        value_type_key: &str,
        value_key: &str,
    ) -> Result<Self, Error> {
        let value_type = req.param(value_type_key)?;
        let value = req.param(value_key)?;

        Reference::try_from_fields(value_type, value)
    }

    pub fn try_from_fields(value_type: &str, value: &'a str) -> Result<Self, Error> {
        match value_type {
            "slug" => {
                tide::log::debug!("Reference via slug, {value}");
                Ok(Reference::Slug(value))
            }
            "id" => {
                tide::log::debug!("Reference via ID, {value}");
                let id = value.parse()?;
                Ok(Reference::Id(id))
            }
            _ => Err(Error::from_str(
                StatusCode::BadRequest,
                "May only specify object by 'id' or 'slug'",
            )),
        }
    }
}

impl From<i64> for Reference<'static> {
    #[inline]
    fn from(id: i64) -> Reference<'static> {
        Reference::Id(id)
    }
}

impl<'a> From<&'a str> for Reference<'a> {
    #[inline]
    fn from(slug: &'a str) -> Reference<'a> {
        Reference::Slug(slug)
    }
}

impl<'a> TryFrom<&'a ApiRequest> for Reference<'a> {
    type Error = Error;

    #[inline]
    fn try_from(req: &'a ApiRequest) -> Result<Reference<'a>, Error> {
        Reference::try_from_fields_key(req, "type", "id_or_slug")
    }
}

impl<'a> From<Reference<'a>> for SerialReference<'a> {
    #[inline]
    fn from(reference: Reference<'a>) -> SerialReference<'a> {
        match reference {
            Reference::Id(id) => SerialReference::Id(id),
            Reference::Slug(slug) => SerialReference::Slug(Cow::from(slug)),
        }
    }
}

impl<'a> SerialReference<'a> {
    pub fn as_reference_struct(
        self: &'a Self,
    ) -> Reference<'a> {
        match self {
            SerialReference::Id(id) => Reference::Id(*id),
            SerialReference::Slug(ref slug) => Reference::Slug(slug),
        }
    }
}