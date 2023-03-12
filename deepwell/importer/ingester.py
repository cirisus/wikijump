import json
import logging
import os
import re
import sqlite3

from py7zr import SevenZipFile

from .structures import *

REVISION_FILENAME_REGEX = re.compile(r"(\d+)\.txt")

SQLITE_SCHEMA = """

-- General note: No foreign key constraints.
--               If there are missing items, instead of needing
--               to re-ingest all child elements, we just add
--               the (then-)invalid IDs and figure it out manually later.

CREATE TABLE IF NOT EXISTS user (
    wikidot_id INTEGER PRIMARY KEY,
    created_at INTEGER NOT NULL,
    full_name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    account_type TEXT NOT NULL,
    karma INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS page (
    wikidot_id INTEGER PRIMARY KEY,
    page_slug TEXT NOT NULL,
    site_slug TEXT NOT NULL,
    tags TEXT NOT NULL,
    locked INTEGER NOT NULL,  -- boolean
    discussion_thread_id INTEGER,

    UNIQUE (site_slug, page_slug)
);

CREATE TABLE IF NOT EXISTS page_revision (
    wikidot_id INTEGER PRIMARY KEY,
    revision_number INTEGER NOT NULL
    page_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    flags TEXT NOT NULL,
    comments TEXT NOT NULL,
    wikitext TEXT NOT NULL,

    UNIQUE (page_id, revision_number)
);

CREATE TABLE IF NOT EXISTS page_vote (
    page_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    value INTEGER NOT NULL,

    UNIQUE (page_id, user_id)
);

CREATE TABLE IF NOT EXISTS file (
    wikidot_id INTEGER PRIMARY KEY,
    page_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    size INTEGER NOT NULL,
    mime TEXT NOT NULL,
    mime_description TEXT NOT NULL,
    internal_version INTEGER NOT NULL,
    data BLOB NOT NULL,

    UNIQUE (page_id, name)
);

-- Nothing collected for forum groups at present

CREATE TABLE IF NOT EXISTS forum_category (
    wikidot_id INTEGER PRIMARY KEY,
    site_slug TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS forum_thread (
    wikidot_id INTEGER PRIMARY KEY,
    forum_category_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    user_id INTEGER,  -- NULL if made by Wikidot
    locked INTEGER NOT NULL,  -- boolean
    sticky INTEGER NOT NULL  -- boolean
);

CREATE TABLE IF NOT EXISTS forum_post (
    wikidot_id INTEGER PRIMARY KEY,
    forum_thread_id INTEGER NOT NULL,
    parent_post_id INTEGER,
    user_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    title TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS forum_post_revision (
    wikidot_id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    title TEXT NOT NULL,
    html TEXT NOT NULL
);
"""

logger = logging.getLogger(__name__)


class Ingester:
    __slots__ = ("directory", "conn", "replace_colons")

    # Init and deinit
    def __init__(self, wikicomma_directory, database_path, replace_colons):
        self.directory = wikicomma_directory
        self.conn = sqlite3.connect(database_path)
        self.replace_colons = replace_colons

    def setup(self):
        with self.conn as cur:
            cur.executescript(SQLITE_SCHEMA)

    def close(self):
        self.conn.close()

    # Helper methods
    def path(self, *parts):
        return os.path.join(self.directory, *parts)

    def read_json(self, *parts):
        path = os.path.join(*parts)

        if not os.path.isabs(path):
            path = self.path(path)

        with open(path) as file:
            return json.load(file)

    def process_filename(self, filename):
        # Wikicomma uses underscores even though
        # colons are valid in UNIX paths.
        if self.replace_colons:
            return filename.replace(":", "_")

        return filename

    def open_revisions(self, site_directory, page_slug):
        filename = self.process_filename(f"{page_slug}.7z")
        path = os.path.join(site_directory, "pages", filename)
        return SevenZipFile(path, "r")

    # Main ingestion methods
    def ingest_users(self):
        logger.info("Ingesting all user data")
        users = []
        users_directory = self.path("_users")

        for filename in os.listdir(users_directory):
            if filename == "pending.json":
                logger.debug("Skipping pending.json file")
                continue

            logger.info("Ingesting users from %s", filename)
            with open(os.path.join(users_directory, filename)) as file:
                users_data = json.load(file)

            # Load user data
            users.clear()
            for _, user_data in users_data:
                users.append(
                    User(
                        full_name=user_data["full_name"],
                        slug=user_data["slug"],
                        created_at=user_data["wikidot_user_since"],
                        account_type=user_data["account_type"],
                        karma=user_data["activity"],
                        wikidot_id=user_data["user_id"],
                    )
                )

            # Insert into database
            rows = [
                (
                    user.wikidot_id,
                    user.created_at,
                    user.full_name,
                    user.slug,
                    user.account_type,
                    user.karma,
                )
                for user in users
            ]

            query = """
            INSERT OR REPLACE INTO user (
                wikidot_id,
                created_at,
                full_name,
                slug,
                account_type,
                karma
            )
            VALUES (?, ?, ?, ?, ?, ?)
            """

            with self.conn as cur:
                cur.executemany(query, rows)

    def ingest_sites(self):
        logger.info("Ingesting all site data")
        for site in os.listdir(self.directory):
            if site == "_users":
                logger.debug("Skipping _users directory")
                continue

            self.ingest_site(site)

    def ingest_site(self, site):
        logger.info("Ingesting site '%s'", site)
        self.ingest_pages(site)
        self.ingest_forums(site)

    def ingest_pages(self, site_slug):
        logger.info("Ingesting all pages for site '%s'", site)
        site_directory = self.path(site_slug)
        page_mapping = self.read_json(site_directory, "meta", "page_id_map.json")
        file_mapping = self.read_json(site_directory, "meta", "file_map.json")
        logger.info("Processing %d pages", len(page_mapping))

        for page_id, page_slug in page_mapping.items():
            logger.info("Ingesting page '%s' (ID %d)", page_slug, page_id)
            page_id = int(page_id)  # JSON keys are always strings
            self.ingest_page(
                site_directory,
                file_mapping,
                site_slug,
                page_id,
                page_slug,
            )

    def ingest_page(self, site_directory, file_mapping, site_slug, page_id, page_slug):
        def read_page_metadata():
            filename = self.process_filename(f"{page_slug}.json")
            metadata = self.read_json(site_directory, "meta", "pages", filename)
            assert metadata["name"] == page_slug, "Path and metadata slug do not match"
            return metadata

        metadata = read_page_metadata()
        page = Page(
            wikidot_id=metadata["page_id"],
            title=metadata.get("title", ""),
            slug=page_slug,
            tags=metadata["tags"],
            locked=metadata["is_locked"],
            discussion_thread_id=-1,  # TODO unknown
        )

        query = """
        INSERT OR REPLACE INTO page (
            wikidot_id,
            page_slug,
            site_slug,
            title,
            tags,
            locked,
            discussion_thread_id
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """

        with self.conn as cur:
            cur.execute(
                query,
                (
                    page.wikidot_id,
                    page_slug,
                    site_slug,
                    page.title,
                    json.dumps(page.tags),
                    page.locked,
                    page.discussion_thread_id,
                ),
            )

        self.ingest_votes(page_id=page.wikidot_id, votes=metadata["votings"])
        self.ingest_revisions(
            site_directory=site_directory,
            page_slug=page_slug,
            page_id=page.wikidot_id,
            revisions=metadata["revisions"],
        )
        self.ingest_files(
            site_directory=site_directory,
            file_mapping=file_mapping,
            page_id=page_id,
            files=metadata["files"],
        )

    def ingest_votes(self, *, page_id, votes):
        logger.info("Adding %d votes for this page", len(votes))

        def convert_vote_value(value):
            if isinstance(value, int):
                return value
            elif isinstance(value, bool):
                return 1 if value else -1
            else:
                raise TypeError(f"Unknown type for vote value: {type(value)}")

        rows = [
            (
                page_id,
                user_id,
                convert_vote_value(value),
            )
            for user_id, value in votes
        ]

        query = """
        INSERT OR REPLACE INTO page_vote (
            page_id,
            user_id,
            value
        ) VALUES (?, ?, ?)
        """

        with self.conn as cur:
            cur.executemany(query, rows)

    def ingest_revisions(self, *, site_directory, page_slug, page_id, revisions):
        logger.info("Adding %d revisions for this page", len(revisions))

        query = """
        INSERT OR REPLACE INTO page_revision (
            wikidot_id,
            revision_number,
            page_id,
            user_id,
            created_at,
            flags,
            comments,
            wikitext
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """

        logger.info("Loading wikitext revisions for this page")
        wikitexts = {}
        with self.open_revisions(site_directory, page_slug) as archive:
            for filename, data in archive.readall().items():
                logger.debug("Found file in archive: %s", filename)
                match = REVISION_FILENAME_REGEX.fullmatch(filename)
                revision_number = int(match[1])
                wikitext = data.read().decode("utf-8")

                wikitexts[revision_number] = wikitext

        logger.info("Inserting revisions for this page")
        for data in revisions:
            revision = PageRevision(
                wikidot_id=data["global_revision"],
                revision_number=data["revision"],
                created_at=data["stamp"],
                flags=data["flags"],
                page_id=page_id,
                user_id=data["author"],
                comments=data["commentary"],
            )

            with self.conn as cur:
                logger.info(
                    "Inserting revision #%d, ID %d",
                    revision.revision_number,
                    revision.wikidot_id,
                )
                cur.execute(
                    query,
                    (
                        revision.wikidot_id,
                        revision.revision_number,
                        revision.page_id,
                        revision.user_id,
                        revision.created_at,
                        revision.flags,
                        revision.comments,
                        wikitexts[revision.wikidot_id],
                    ),
                )

    def ingest_files(self, *, site_directory, file_mapping, page_id, files):
        logger.info("Ingesting files for this page")

        query = """
        INSERT OR REPLACE INTO file (
            wikidot_id,
            page_id,
            user_id,
            created_at,
            name,
            url,
            size,
            mime,
            mime_description,
            internal_version,
            data
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """

        def load_file(model):
            logger.debug("Loading file data for %s", model.name)
            file_location = file_mapping(str(model.wikidot_id))
            file_path = os.path.join(site_directory, "files", file_location["path"])
            assert (
                model.url == file_location["url"]
            ), "File entry and mapping URLs do not match"

            with open(file_path, "rb") as file:
                return file.read()

        for data in files:
            file = File(
                wikidot_id=data["file_id"],
                name=data["name"],
                url=data["url"],
                size=data["size_bytes"],
                mime=data["mime"],
                mime_description=data["content"],
                page_id=page_id,
                user_id=data["author"],
                created_at=data["stamp"],
                internal_version=data["internal_version"],
            )
            file_data = load_file(file)

            logger.info("Inserting file data for ID %d", file.wikidot_id)

            with self.conn as cur:
                cur.execute(
                    query,
                    (
                        file.wikidot_id,
                        file.page_id,
                        file.user_id,
                        file.created_at,
                        file.name,
                        file.url,
                        file.size,
                        file.mime,
                        file.mime_description,
                        file.internal_version,
                        file_data,
                    ),
                )

    def ingest_forums(self, site_slug):
        logger.info("Ingesting all forum data for site '%s'", site_slug)
        site_directory = self.path(site_slug)
        meta_directory = os.path.join(site_directory, "meta", "forum")
        forum_directory = os.path.join(site_directory, "forum")

        self.ingest_forum_categories(site_slug, meta_directory)
        self.ingest_forum_threads(site_slug, meta_directory, forum_directory)

    def ingest_forum_categories(self, site_slug, meta_directory):
        logger.info("Ingesting forum categories for site '%s'", site_slug)
        directory = os.path.join(meta_directory, "category")

        categories = []
        for path in os.listdir(directory):
            data = self.read_json(directory, path)
            categories.append(
                ForumCategory(
                    wikidot_id=data["id"],
                    title=data["title"],
                    description=data["description"],
                    posts=data["posts"],
                    threads=data["threads"],
                    last_user=data["lastUser"],
                    full_scan=data["full_scan"],
                    last_page=data["last_page"],
                )
            )

        query = """
        INSERT OR REPLACE INTO forum_category (
            wikidot_id,
            site_slug,
            title,
            description
        ) VALUES (?, ?, ?, ?)
        """

        rows = [
            (category.wikidot_id, site_slug, category.title, category.description)
            for category in categories
        ]

        with self.conn as cur:
            logger.info("Inserting %d forum categories", len(rows))
            cur.executemany(query, rows)

    def ingest_forum_threads(self, site_slug, meta_directory, forum_directory):
        logger.info("Ingesting forum threads for site '%s'", site_slug)
        for category in os.listdir(meta_directory):
            if category == "category":
                logger.debug("Skipping categories directory")
                continue

            category_id = int(category)
            thread_directory = os.path.join(meta_directory, category)
            for thread in os.listdir(thread_directory):
                thread_data = self.read_json(thread_directory, thread)
                self.ingest_forum_thread(category_id, thread_data)

    def ingest_forum_thread(self, forum_category_id, data):
        logger.info("Ingesting forum thread")

        thread = ForumThread(
            wikidot_id=data["id"],
            forum_category_id=forum_category_id,
            created_at=data["started"],
            user_id=data["startedUser"],
            title=data["title"],
            description=data["description"],
            posts=data["postsNum"],
            last_post_time=data.get("last"),
            last_post_user=data.get("lastUser"),
            locked=-1,  # TODO
            sticky=data["sticky"],
        )

        query = """
        INSERT OR REPLACE INTO forum_thread (
            wikidot_id,
            forum_category_id,
            title,
            description,
            user_id,
            locked,
            sticky
        ) VALUES (?, ?, ?, ?, ?)
        """

        with self.conn as cur:
            logger.info("Inserting forum thread ID %d", thread.wikidot_id)
            cur.execute(
                query,
                (
                    thread.wikidot_id,
                    thread.forum_category_id,
                    thread.title,
                    thread.description,
                    thread.user_id,
                    thread.locked,
                    thread.sticky,
                ),
            )

        self.ingest_forum_thread_posts(data["posts"])

    def ingest_forum_thread_posts(self, forum_thread_id, posts):
        logger.debug("Ingesting forum posts from thread ID %d", forum_thread_id)

        for post in posts:
            self.ingest_forum_thread_post(forum_thread_id, None, post)

    def ingest_forum_thread_post(self, forum_thread_id, parent_post_id, data):
        logger.info("Ingesting forum post data")

        post = ForumPost(
            wikidot_id=data["id"],
            forum_thread_id=forum_thread_id,
            parent_post_id=parent_post_id,
            title=data["title"],
            created_at=data["stamp"],
            user_id=data["poster"],
            last_edit_at=data.get("lastEdit"),
            last_edit_by=data.get("lastEditBy"),
        )

        # Insert into database
        query = """
        INSERT OR REPLACE INTO forum_post (
            wikidot_id,
            forum_thread_id,
            parent_post_id,
            user_id,
            created_at,
            title
        ) VALUES (?, ?, ?, ?, ?, ?)
        """

        with self.conn as cur:
            logger.info("Inserting forum thread post ID %d", post.wikidot_id)
            cur.execute(
                query,
                (
                    post.wikidot_id,
                    post.forum_thread_id,
                    post.user_id,
                    post.created_at,
                    post.title,
                ),
            )

        # Recursively go through children
        # (This will not be endless because there is a max forum depth,
        # and the overall graph is acyclic.)
        for child_post in data["children"]:
            self.ingest_forum_thread_post(forum_thread_id, post.wikidot_id, child_post)

        # Add revisions for this post
        #
        # This means children *revisions* are inserted before the
        # parent, but that's fine since the main row itself is
        # added in correct order.
        #
        # (That said we don't have foreign key constraints so this
        # difference is mostly academic...)
        self.ingest_forum_thread_revisions(data["revisions"])

    def ingest_forum_thread_revisions(self, revision_data):
        # NOTE: IMPORTANT
        #       If there is only 1 revision (i.e. post has not been edited)
        #       then revisions is EMPTY
        #
        #       need to extract and fetch from 7z for all
        #
        #       sort by ID? get revision_number from that
        logger.error("TODO: ingest_forum_thread_revisions")

    def ingest_forum_thread_revision(self):
        logger.error("TODO: ingest_forum_thread_revision")
        # TODO
