from typing import List
import marshmallow_dataclass

from ideascale_importer import utils


class DrepAttributes:
    ...


class Drep:
    id: int
    attributes: DrepAttributes


DrepAttributesSchema = marshmallow_dataclass.class_schema(DrepAttributes)
DrepSchema = marshmallow_dataclass.class_schema(Drep)


class Client:
    def __init__(self, api_url: str):
        self.inner = utils.JsonHttpClient(api_url)

    async def dreps(self) -> List[Drep]:
        res = await self.inner.get("/api/dreps")
        return DrepSchema().load(res, many=True) or []
