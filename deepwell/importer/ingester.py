import logging
import os
import re
import sqlite3

from .structures import *

from py7zr import SevenZipFile

REVISION_FILENAME_REGEX = re.compile(r"(\d+)\.txt")

SQLITE_SCHEMA = """
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
    locked INTEGER NOT NULL,
    discussion_thread_id INTEGER,

    UNIQUE (site_slug, page_slug)
);

CREATE TABLE IF NOT EXISTS page_revision (
    wikidot_id INTEGER PRIMARY KEY,
    revision_number INTEGER NOT NULL
    page_id INTEGER NOT NULL REFERENCES page(wikidot_id),
    user_id INTEGER NOT NULL REFERENCES user(wikidot_id),
    created_at INTEGER NOT NULL,
    flags TEXT NOT NULL,
    comments TEXT NOT NULL,
    wikitext TEXT NOT NULL,

    UNIQUE (page_id, revision_number)
);

CREATE TABLE IF NOT EXISTS page_vote (
    page_id INTEGER NOT NULL REFERENCES page(wikidot_id),
    user_id INTEGER NOT NULL REFERENCES user(wikidot_id),
    value INTEGER NOT NULL,

    UNIQUE (page_id, user_id)
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

        for filename in os.listdir(self.path("_users")):
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
            INSERT INTO user (
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

            self.ingest_site(self, site)

    def ingest_site(self, site):
        logger.info("Ingesting site '%s'", site)
        self.ingest_pages(self, site)

    def ingest_pages(self, site_slug):
        logger.info("Ingesting all pages from site")
        site_directory = self.path(site_slug)
        page_mapping = self.read_json(site_directory, "meta", "page_id_map.json")
        file_mapping = self.read_json(site_directory, "meta", "file_map.json")
        logger.info("Processing %d pages", len(page_mapping))

        for page_id, page_slug in page_mapping.items():
            logger.info("Ingesting page '%s' (ID %d)", page_slug, page_id)
            page_id = int(page_id)  # JSON keys are always strings
            self.ingest_page(site_directory, site_slug, page_id, page_slug)

    def ingest_page(self, site_directory, site_slug, page_id, page_slug):
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
        INSERT INTO page (
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

        self.ingest_votes(page.wikidot_id, metadata["votings"])
        self.ingest_files(page.wikidot_id, site_slug, metadata["files"])
        self.ingest_revisions(
            page.wikidot_id,
            site_directory,
            page_slug,
            page.wikidot_id,
            metadata["revisions"],
        )

    def ingest_votes(self, page_id, votes):
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
        INSERT INTO page_vote (
            page_id,
            user_id,
            value
        ) VALUES (?, ?, ?)
        """

        with self.conn as cur:
            cur.executemany(query, rows)

    def ingest_revisions(self, site_directory, page_slug, page_id, revisions):
        logger.info("Adding %d revisions for this page", len(revisions))

        query = """
        INSERT INTO page_revision (
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
                wikitext=wikitexts[data["revision"]],
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
                        revision.created_at,
                        revision.flags,
                        revision.page_id,
                        revision.user_id,
                        revision.wikitext,
                        revision.comments,
                    ),
                )

    def ingest_files(self, page_id, site_slug, files):
        # TODO
        logger.error("ingest_files() not implemented")
