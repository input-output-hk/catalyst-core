import re
from typing import Any, Dict, List, TypeVar


DictOrList = TypeVar("DictOrList", Dict[str, Any], List[Any])


def snake_case_keys(x: DictOrList):
    """
    Recursively transforms all dict keys to snake_case.
    """

    if isinstance(x, dict):
        keys = list(x.keys())
        for k in keys:
            v = x.pop(k)
            snake_case_keys(v)
            x[snake_case(k)] = v
    elif isinstance(x, list):
        for i in range(len(x)):
            snake_case_keys(x[i])


def snake_case(s: str) -> str:
    """
    Transforms a string to snake_case.
    """

    return re.sub(r"([a-z])([A-Z])", r"\1_\2", s).lower()
