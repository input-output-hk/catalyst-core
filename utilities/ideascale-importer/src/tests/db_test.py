import dataclasses
import unittest
from typing import Optional

import db


class TestDb(unittest.TestCase):
    @dataclasses.dataclass
    class TestModel(db.Model):
        x: int
        s: str
        j: Optional[int]
        k: Optional[str]

        @staticmethod
        def table() -> str:
            return "test_table"

    def test_insert_statement(self):
        s = db.insert_statement(TestDb.TestModel(1, "somestr", None, "notnone"))
        self.assertEqual(s, "INSERT INTO test_table (x,s,k) VALUES (1,'somestr','notnone')")


if __name__ == "__main__":
    unittest.main()
