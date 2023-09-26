from ideascale_importer import utils


def test_snake_case_keys_on_dicts():
        d = {
            "A": 1,
            "SnakeCase": {
                "InnerDict": 2,
                "some_key": 3,
            },
            "list": [1, {"otherKey": [1, 2, {"SomeKey": 1}]}],
        }

        expected_d = {
            "a": 1,
            "snake_case": {
                "inner_dict": 2,
                "some_key": 3,
            },
            "list": [1, {"other_key": [1, 2, {"some_key": 1}]}],
        }

        utils.snake_case_keys(d)

        assert d == expected_d
