import sqlite3
import os


class Index:
    """Represents a local index of fragments.

    Attributes:
        db_path: The path to the index database
    """

    def __init__(self, path: str):
        self.db_path = os.path.join(path, "index.db")
        self._init_db()

    def _init_db(self):
        """Initializes the index database."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
            CREATE TABLE IF NOT EXISTS fragments (
                fragment_id TEXT PRIMARY KEY
            )
            """
            )
            conn.execute(
                """
            CREATE TABLE IF NOT EXISTS node_stats (
                key TEXT PRIMARY KEY,
                value INTEGER
            )
            """
            )
            conn.commit()

    def insert(self, fragment_id: str):
        """Inserts a fragment into the index.

        Args:
            fragment_id: The ID of the fragment to insert
        """
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
            INSERT OR IGNORE INTO fragments (fragment_id) VALUES (?)
            """,
                (fragment_id,),
            )
            conn.commit()

    def exists(self, fragment_id: str) -> bool:
        """Checks if a fragment exists in the index.

        Args:
            fragment_id: The ID of the fragment to check

        Returns:
            True if the fragment exists, False otherwise
        """
        with sqlite3.connect(self.db_path) as conn:
            cur = conn.execute(
                """
            SELECT 1 FROM fragments WHERE fragment_id = ?
            """,
                (fragment_id,),
            )
            return cur.fetchone() is not None

    def set_last_block_height(self, height: int):
        """Sets the last block height in the index.

        Args:
            height: The last block height
        """
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
            INSERT OR REPLACE INTO node_stats
            (key, value)
            VALUES
            ('last_block_height', ?)
            """,
                (height,),
            )
            conn.commit()

    def get_last_block_height(self) -> int:
        """Gets the last block height from the index.

        Returns:
            The last block height
        """
        with sqlite3.connect(self.db_path) as conn:
            cur = conn.execute(
                """
            SELECT value FROM node_stats WHERE key = 'last_block_height'
            """
            )
            result = cur.fetchone()
            return result[0] if result else 0

    def purge(self):
        """Purges the index."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
            DELETE FROM fragments
            """
            )
            conn.commit()
