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

    def ingest_users(self):
        logger.info("Ingesting all user data")
        users = load_users(os.path.join(self.directory, "_users"))
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
                # Special directory for user information
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
