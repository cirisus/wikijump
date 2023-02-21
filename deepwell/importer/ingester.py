import sqlite3

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
    __slots__ = (
        "conn",
    )

    def __init__(self, database_path):
        self.conn = sqlite3.connect(database_path)

    def seed(self):
        self.conn.executescript(SQLITE_SCHEMA)

    def close(self):
        self.conn.close()
