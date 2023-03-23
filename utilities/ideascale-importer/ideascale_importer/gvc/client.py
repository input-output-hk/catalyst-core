from pydantic.dataclasses import dataclass
import pydantic.tools
from typing import List

from ideascale_importer import utils


@dataclass
class DrepAttributes:
    ...


@dataclass
class Drep:
    id: int
    attributes: DrepAttributes


class Client:
    def __init__(self, api_url: str):
        self.inner = utils.JsonHttpClient(api_url)

    async def dreps(self) -> List[Drep]:
        res = await self.inner.get("/api/dreps")
        return [pydantic.tools.parse_obj_as(Drep, e) for e in res]
