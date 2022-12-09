import re
from typing import Dict, TypeVar

T = TypeVar("T")

def snake_case_keys(d: Dict[str, T]) -> Dict[str, T]:
    new_d = {}
    for k, v in d.items():
        if isinstance(v, dict):
            v = snake_case_keys(v)

        k = re.sub(r"([a-z])([A-Z])", r"\1_\2", k).lower()

        new_d[k] = v

    return new_d
