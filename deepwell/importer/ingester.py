import logging
import os
import sqlite3

from .user import load_users

SQLITE_SCHEMA = """
CREATE TABLE IF NOT EXISTS user (
    wikidot_id INTEGER PRIMARY KEY,
    created_at INTEGER NOT NULL,
    full_name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    account_type TEXT NOT NULL,
    karma INTEGER NOT NULL
);
"""

logger = logging.getLogger(__name__)


class Ingester:
    __slots__ = ("conn",)

    def __init__(self, wikicomma_directory, database_path):
        self.directory = wikicomma_directory
        self.conn = sqlite3.connect(database_path)

    def setup(self):
        with self.conn as cur:
            cur.executescript(SQLITE_SCHEMA)

    def close(self):
        self.conn.close()

    def path(self, *parts):
        return os.path.join(self.directory, *parts)

    def ingest_users(self):
        logger.info("Ingesting all user data")
        users = load_users()
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

    def load_users(self):
        users = []

        for filename in os.listdir(self.path("_users")):
            if filename == "pending.json":
                logger.debug("Skipping pending.json file")
                continue

            logger.info("Loading users from %s", filename)
            with open(os.path.join(users_directory, filename)) as file:
                users_data = json.load(file)

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

        return users

    def ingest_sites(self):
        logger.info("Ingesting all site data")
        for site in os.listdir(self.directory):
            if site == "_users":
                logger.debug("Skipping _users directory")
                continue

            self.ingest_site(self, site)

    def ingest_site(self, site):
        logger.info("Ingesting site '%s'", site)
        site_directory = os.path.join(self.directory, site)
        self.ingest_pages(self, site_directory)
        self.ingest_files(self, site_directory)

    def ingest_pages(self, site_directory):
        logger.info("Ingesting all pages from site")
        # TODO

    def ingest_files(self, site_directory):
        logger.info("Ingesting all files from site")
        # TODO
