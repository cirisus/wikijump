/*
 * services/revision/service.rs
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
use crate::json_utils::{json_to_string_list, string_list_to_json};
use crate::models::page_revision::{
    self, Entity as PageRevision, Model as PageRevisionModel,
};
use crate::services::render::RenderOutput;
use crate::services::{
    LinkService, OutdateService, RenderService, SiteService, TextService,
};
use crate::web::{split_category, split_category_name, RevisionDirection};
use ftml::data::PageInfo;
use ftml::settings::{WikitextMode, WikitextSettings};
use ref_map::*;
use std::borrow::Cow;
use std::num::NonZeroI32;

macro_rules! cow {
    ($s:expr) => {
        Cow::Borrowed($s.as_ref())
    };
}

macro_rules! cow_opt {
    ($s:expr) => {
        $s.ref_map(|s| cow!(s))
    };
}

macro_rules! conditional_future {
    ($conditional:expr, $future:expr $(,)?) => {
        async move {
            if $conditional {
                $future.await
            } else {
                Ok(())
            }
        }
    };
}

#[derive(Debug)]
pub struct RevisionService;

impl RevisionService {
    /// Creates a new revision.
    ///
    /// For the given page, look at the changes to make. If there are none,
    /// or they are all equivalent to the previous revision's, then no
    /// revision is committed and `Ok(None)` is returned.
    ///
    /// If there are changes, then the new revision is created and all the
    /// appropriate updating is done. For instance, recompiling the page
    /// or updating backlinks.
    ///
    /// For page renames, this does not explicitly check if the target slug
    /// already exists. If so, the database will fail with a uniqueness error.
    /// This is checked in `PageService::rename()`, where renames should be done from.
    ///
    /// The revision number is subject to an invariant:
    /// * For a new page, then the value must be `0` (this corresponds with a `previous` of `None`).
    /// * For an existing page, then the value must be precisely one greater than the previous
    ///   revision's number. No holes are permitted in the revision count, for some maximum
    ///   revision number `n`, there must be revisions for each revision number from `0` to `n`
    ///   inclusive.
    ///
    /// This is enforced by requiring the previous revision be passed in during creation.
    ///
    /// # Panics
    /// If the given previous revision is for a different page or site, this method will panic.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        CreateRevision {
            user_id,
            comments,
            body,
        }: CreateRevision,
        previous: PageRevisionModel,
    ) -> Result<Option<CreateRevisionOutput>> {
        let txn = ctx.transaction();

        // Get the new revision number and the change tasks to process
        let (tasks, revision_number) = {
            // Check for basic consistency
            assert_eq!(
                previous.site_id, site_id,
                "Previous revision has an inconsistent site ID",
            );
            assert_eq!(
                previous.page_id, page_id,
                "Previous revision has an inconsistent page ID",
            );

            // Check to see if any fields have changed
            let tasks = RevisionTasks::determine(&previous, &body);
            if tasks.is_empty() {
                tide::log::info!("No changes from previous revision");
                return Ok(None);
            }

            // Can proceed, increment from previous
            (tasks, previous.revision_number + 1)
        };

        // Fields to create in the revision
        let mut parser_warnings = None;
        let mut old_slug = None;
        let mut changes = Vec::new();
        let PageRevisionModel {
            mut wikitext_hash,
            mut compiled_hash,
            mut compiled_at,
            mut compiled_generator,
            hidden,
            mut title,
            mut alt_title,
            mut slug,
            mut tags,
            metadata, // TODO make mut, when we start modifying this
            ..
        } = previous;

        // Update fields from input
        if let ProvidedValue::Set(new_title) = body.title {
            changes.push(str!("title"));
            title = new_title;
        }

        if let ProvidedValue::Set(new_alt_title) = body.alt_title {
            changes.push(str!("alt_title"));
            alt_title = new_alt_title;
        }

        if let ProvidedValue::Set(new_slug) = body.slug {
            changes.push(str!("slug"));
            old_slug = Some(slug);
            slug = new_slug;
        }

        if let ProvidedValue::Set(new_tags) = body.tags {
            changes.push(str!("tags"));
            tags = string_list_to_json(&new_tags)?;
        }

        // Get slug strings for the new location
        let (category_slug, page_slug) = split_category_name(&slug);

        // Get wikitext, set wikitext hash
        let wikitext = match body.wikitext {
            // Insert new wikitext and update hash
            ProvidedValue::Set(wikitext) => {
                changes.push(str!("wikitext"));
                let new_hash = TextService::create(ctx, wikitext.clone()).await?;
                replace_hash(&mut wikitext_hash, &new_hash);
                wikitext
            }

            // Use previous revision's wikitext
            ProvidedValue::Unset => TextService::get(ctx, &wikitext_hash).await?,
        };

        // Run tasks based on changes:
        // See RevisionTasks struct for more information.

        if tasks.render_and_update_links {
            // This is necessary until we are able to replace the
            // 'tags' column with TEXT[] instead of JSON.
            let temp_tags = json_to_string_list(tags.clone())?;
            let render_input = RenderPageInfo {
                slug: &slug,
                title: &title,
                alt_title: alt_title.ref_map(|s| s.as_str()),
                rating: 0.0, // TODO
                tags: &temp_tags,
            };

            // Run renderer and related tasks
            //
            // Since outdating depends on scope (see RevisionTasks),
            // we don't do that right after here.
            //
            // TODO: use html_output
            let render_output = Self::render_and_update_links(
                ctx,
                site_id,
                page_id,
                wikitext,
                render_input,
            )
            .await?;

            // Update fields
            parser_warnings = Some(render_output.warnings);
            replace_hash(&mut compiled_hash, &render_output.compiled_hash);
            compiled_generator = render_output.compiled_generator;
            compiled_at = now();
        }

        // Perform outdating based on changes made.
        match old_slug {
            Some(ref old_slug) => {
                // If there's an "old slug" set, then this is a page rename / move.
                // Thus we should invoke the OutdateService for both the source
                // and destination.
                //
                // This is equivalent to the three outdate calls below, but for
                // the source and destination slugs, which is why we don't
                // also run those again.

                OutdateService::process_page_move(ctx, site_id, page_id, old_slug, &slug)
                    .await?;
            }
            None => {
                // Run all outdating tasks in parallel.
                //
                // This macro runs the given method (second value) if the condition (first value)
                // is true, otherwise does nothing.

                try_join!(
                    conditional_future!(
                        tasks.rerender_incoming_links,
                        OutdateService::outdate_incoming_links(ctx, site_id, page_id),
                    ),
                    conditional_future!(
                        tasks.rerender_outgoing_includes,
                        OutdateService::outdate_outgoing_includes(ctx, site_id, page_id),
                    ),
                    conditional_future!(
                        tasks.rerender_templates,
                        OutdateService::outdate_templates(
                            ctx,
                            site_id,
                            category_slug,
                            page_slug,
                        ),
                    ),
                )?;
            }
        }

        // Insert the new revision into the table
        let changes = string_list_to_json(&changes)?;
        let model = page_revision::ActiveModel {
            revision_number: Set(revision_number),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            changes: Set(changes),
            wikitext_hash: Set(wikitext_hash),
            compiled_hash: Set(compiled_hash),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            comments: Set(comments),
            hidden: Set(hidden),
            title: Set(title),
            alt_title: Set(alt_title),
            slug: Set(slug),
            tags: Set(tags),
            metadata: Set(metadata),
            ..Default::default()
        };

        let PageRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(Some(CreateRevisionOutput {
            revision_id,
            revision_number,
            parser_warnings,
        }))
    }

    /// Creates the first revision for a newly-inserted page.
    ///
    /// The first revision of a page is special.
    /// A revision change cannot be missing any fields (since there is
    /// not a previous revision to take prior data from), and always
    /// inserts, since it's not possible for it to be an empty revision
    /// (since there's no prior revision for it to be equal to).
    pub async fn create_first(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        CreateFirstRevision {
            user_id,
            comments,
            wikitext,
            title,
            alt_title,
            slug,
        }: CreateFirstRevision,
    ) -> Result<CreateFirstRevisionOutput> {
        let txn = ctx.transaction();

        // Add wikitext
        let wikitext_hash = TextService::create(ctx, wikitext.clone()).await?;

        // Render first revision
        let render_input = RenderPageInfo {
            slug: &slug,
            title: &title,
            alt_title: alt_title.ref_map(|s| s.as_str()),
            rating: 0.0, // TODO
            tags: &[],   // Initial revision always has empty tags
        };

        let RenderOutput {
            // TODO: use html_output
            html_output: _,
            warnings,
            compiled_hash,
            compiled_generator,
        } = Self::render_and_update_links(ctx, site_id, page_id, wikitext, render_input)
            .await?;

        // Run outdater
        OutdateService::process_page_displace(ctx, site_id, page_id, &slug).await?;

        // Effective constant, number of changes for the first revision.
        // The first revision is always considered to have changed everything.
        let all_changes = serde_json::json!([
            "wikitext",
            "title",
            "alt_title",
            "slug",
            "tags",
            "metadata"
        ]);

        // Insert the new revision into the table
        let model = page_revision::ActiveModel {
            revision_number: Set(0),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            changes: Set(all_changes),
            wikitext_hash: Set(wikitext_hash.to_vec()),
            compiled_hash: Set(compiled_hash.to_vec()),
            compiled_at: Set(now()),
            compiled_generator: Set(compiled_generator),
            comments: Set(comments),
            hidden: Set(serde_json::json!([])),
            title: Set(title),
            alt_title: Set(alt_title),
            slug: Set(slug),
            tags: Set(serde_json::json!([])),
            metadata: Set(serde_json::json!({})),
            ..Default::default()
        };

        let PageRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(CreateFirstRevisionOutput {
            revision_id,
            parser_warnings: warnings,
        })
    }

    /// Creates a revision marking a page as deleted.
    ///
    /// This revision is called a "tombstone" in that
    /// its only purpose is to mark that the page has been deleted.
    pub async fn create_tombstone(
        _ctx: &ServiceContext<'_>,
        _site_id: i64,
        _page_id: i64,
        _user_id: i64,
        _comments: String,
    ) -> Result<CreateRevisionOutput> {
        // TODO modify metadata field to add 'deleted: true'

        todo!()
    }

    /// Creates a revision marking a pages as restored (i.e., undeleted).
    ///
    /// Similar to `create_tombstone`, this method creates
    /// a revision whose only purpose is to mark that the page
    /// has been restored.
    pub async fn create_resurrection(
        _ctx: &ServiceContext<'_>,
        _site_id: i64,
        _page_id: i64,
        _user_id: i64,
        _slug: String,
        _comments: String,
    ) -> Result<CreateRevisionOutput> {
        // TODO modify metadata field to add 'deleted: false'

        // TODO run undeletion outdater procdures:
        //      - rerender incoming links
        //      - rerender included pages
        //      - update outgoing links
        //      - process nav pages
        //      - process template pages

        todo!()
    }

    /// Helper method for performing rendering for a revision.
    ///
    /// Makes all the changes associated with rendering, such as
    /// committing the new wikitext, calling ftml, and updating
    /// backlinks.
    async fn render_and_update_links(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        wikitext: String,
        RenderPageInfo {
            slug,
            title,
            alt_title,
            rating,
            tags,
        }: RenderPageInfo<'_>,
    ) -> Result<RenderOutput> {
        // Get site
        let site = SiteService::get(ctx, Reference::from(site_id)).await?;

        // Set up parse context
        let settings = WikitextSettings::from_mode(WikitextMode::Page);
        let (category_slug, page_slug) = split_category(slug);
        let page_info = PageInfo {
            page: cow!(page_slug),
            category: cow_opt!(category_slug),
            site: cow!(&site.slug),
            title: cow!(title),
            alt_title: cow_opt!(alt_title),
            rating,
            tags: tags.iter().map(|s| cow!(s)).collect(),
            language: cow!(&site.language),
        };

        // Parse and render
        let output = RenderService::render(ctx, wikitext, &page_info, &settings).await?;

        // Update backlinks
        LinkService::update(ctx, site_id, page_id, &output.html_output.backlinks).await?;

        Ok(output)
    }

    /// Re-renders a page.
    ///
    /// This fetches the latest revision for a page, and re-renders it.
    pub async fn rerender(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let revision = Self::get_latest(ctx, site_id, page_id).await?;
        let wikitext = TextService::get(ctx, &revision.wikitext_hash).await?;

        // This is necessary until we are able to replace the
        // 'tags' column with TEXT[] instead of JSON.
        let temp_tags = json_to_string_list(revision.tags)?;
        let render_input = RenderPageInfo {
            slug: &revision.slug,
            title: &revision.title,
            alt_title: revision.alt_title.ref_map(|s| s.as_str()),
            rating: 0.0, // TODO
            tags: &temp_tags,
        };

        // TODO use html_output
        let RenderOutput {
            compiled_hash,
            compiled_generator,
            ..
        } = Self::render_and_update_links(ctx, site_id, page_id, wikitext, render_input)
            .await?;

        // Update descendents
        OutdateService::process_page_edit(ctx, site_id, page_id, &revision.slug).await?;

        let model = page_revision::ActiveModel {
            revision_id: Set(revision.revision_id),
            compiled_hash: Set(compiled_hash.to_vec()),
            compiled_generator: Set(compiled_generator),
            ..Default::default()
        };

        model.update(txn).await?;
        Ok(())
    }

    /// Modifies an existing revision.
    ///
    /// Normally you should think of revisions as being immutable
    /// entries in an append-only log. This however is not always
    /// true. In addition to `rerender()`, staff are able to change
    /// the `hidden` column, causing some fields of the revision to be hidden,
    /// for instance, if it contains spam, abuse, or harassment.
    pub async fn update(
        ctx: &ServiceContext<'_>,
        revision_id: i64,
        UpdateRevision { user_id, hidden }: UpdateRevision,
    ) -> Result<()> {
        // TODO: record revision edit in audit log
        let _ = user_id;

        let txn = ctx.transaction();
        let hidden = string_list_to_json(&hidden)?;
        let model = page_revision::ActiveModel {
            revision_id: Set(revision_id),
            hidden: Set(hidden),
            ..Default::default()
        };

        // Update and return
        model.update(txn).await?;
        Ok(())
    }

    pub async fn get_latest(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<PageRevisionModel> {
        // NOTE: There is no optional variant of this method,
        //       since all extant pages must have at least one revision.

        let txn = ctx.transaction();
        let revision = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id)),
            )
            .order_by_desc(page_revision::Column::RevisionNumber)
            .one(txn)
            .await?
            .ok_or(Error::NotFound)?;

        Ok(revision)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
    ) -> Result<Option<PageRevisionModel>> {
        let txn = ctx.transaction();
        let revision = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id))
                    .add(page_revision::Column::RevisionNumber.eq(revision_number)),
            )
            .one(txn)
            .await?;

        Ok(revision)
    }

    #[inline]
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
    ) -> Result<bool> {
        Self::get_optional(ctx, site_id, page_id, revision_number)
            .await
            .map(|revision| revision.is_some())
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
    ) -> Result<PageRevisionModel> {
        match Self::get_optional(ctx, site_id, page_id, revision_number).await? {
            Some(revision) => Ok(revision),
            None => Err(Error::NotFound),
        }
    }

    pub async fn count(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<NonZeroI32> {
        let txn = ctx.transaction();
        let row_count = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id)),
            )
            .count(txn)
            .await?;

        // We store revision_number in INT, which is i32.
        // So even though this row count is usize, it
        // should always fit inside an i32.
        let row_count = i32::try_from(row_count)
            .expect("Revision row count greater than revision_number integer size");

        // All pages have at least one revision, so if there are none
        // that means this page does not exist, and we should return an error.
        match NonZeroI32::new(row_count) {
            Some(count) => Ok(count),
            None => Err(Error::NotFound),
        }
    }

    pub async fn get_range(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
        revision_direction: RevisionDirection,
        revision_limit: u64,
    ) -> Result<Vec<PageRevisionModel>> {
        let revision_condition = {
            use page_revision::Column::RevisionNumber;

            // Allow specifying "-1" to mean "the most recent revision",
            // otherwise keep as-is.
            let revision_number = if revision_number >= 0 {
                revision_number
            } else {
                i32::MAX
            };

            // Get correct database condition based on requested ordering
            match revision_direction {
                RevisionDirection::Before => RevisionNumber.lte(revision_number),
                RevisionDirection::After => RevisionNumber.gte(revision_number),
            }
        };

        let txn = ctx.transaction();
        let revisions = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id))
                    .add(revision_condition),
            )
            .order_by_asc(page_revision::Column::RevisionNumber)
            .limit(revision_limit)
            .all(txn)
            .await?;

        Ok(revisions)
    }
}

#[derive(Debug)]
struct RenderPageInfo<'a> {
    slug: &'a str,
    title: &'a str,
    alt_title: Option<&'a str>,
    rating: f32,
    tags: &'a [String],
}

#[inline]
fn replace_hash(dest: &mut Vec<u8>, src: &[u8]) {
    debug_assert_eq!(
        dest.len(),
        src.len(),
        "Lengths of hash buffers are not equal",
    );

    dest.as_mut_slice().copy_from_slice(src);
}
