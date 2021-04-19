/*
 * render/html/render.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use super::context::HtmlContext;
use super::element::{render_element, render_elements};
use crate::tree::Element;
use crate::log::prelude::*;

pub trait ItemRender {
    fn render(&self, log: &Logger, ctx: &mut HtmlContext);
}

impl ItemRender for &'_ str {
    #[inline]
    fn render(&self, _log: &Logger, ctx: &mut HtmlContext) {
        ctx.push_escaped(self);
    }
}

impl ItemRender for &'_ Element<'_> {
    #[inline]
    fn render(&self, log: &Logger, ctx: &mut HtmlContext) {
        render_element(log, ctx, self)
    }
}

impl ItemRender for &'_ [Element<'_>] {
    #[inline]
    fn render(&self, log: &Logger, ctx: &mut HtmlContext) {
        render_elements(log, ctx, self)
    }
}
