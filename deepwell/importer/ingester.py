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


class Ingester:
    __slots__ = ("conn",)

    def __init__(self, wikicomma_directory, database_path):
        self.directory = wikicomma_directory
        self.conn = sqlite3.connect(database_path)

    def setup(self):
        cur = self.conn.cursor()
        cur.executescript(SQLITE_SCHEMA)

    def close(self):
        self.conn.close()

    # Private helper methods
    def section(self, *parts):
        return os.path.join(self.directory, *parts)

    # Object ingestion
    def ingest_users(self):
        users = load_users(self.section("_users"))
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

        cur = self.conn.cursor()
        cur.executemany(query, rows)

    # TODO
