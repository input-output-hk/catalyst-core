import unittest

from .. import utils

class TestUtils(unittest.TestCase):
    def test_snake_case_keys(self):
        d = {
            "A": 1,
            "SnakeCase": {
                "InnerDict": 2,
                "some_key": 3,
            }
        }

        expected_d = {
            "a": 1,
            "snake_case": {
                "inner_dict": 2,
                "some_key": 3,
            }
        }

        self.assertEqual(utils.snake_case_keys(d), expected_d)
