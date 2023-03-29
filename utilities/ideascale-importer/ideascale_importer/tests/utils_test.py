import unittest

from ideascale_importer import utils


class TestUtils(unittest.TestCase):
    def test_snake_case_keys_on_dicts(self):
        d = {
            "A": 1,
            "SnakeCase": {
                "InnerDict": 2,
                "some_key": 3,
            },
            "list": [
                1,
                {
                    "otherKey": [1, 2, {"SomeKey": 1}]
                }
            ]
        }

        expected_d = {
            "a": 1,
            "snake_case": {
                "inner_dict": 2,
                "some_key": 3,
            },
            "list": [
                1,
                {
                    "other_key": [1, 2, {"some_key": 1}]
                }
            ]
        }

        utils.snake_case_keys(d)

        self.assertEqual(d, expected_d)


if __name__ == "__main__":
    unittest.main()
