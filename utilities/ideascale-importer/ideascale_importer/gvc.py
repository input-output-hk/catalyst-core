"""GVC API module."""
from pydantic import BaseModel
from typing import List

from ideascale_importer import utils


class DrepAttributes(BaseModel):
    """Represents DREP attributes from the GVC API."""

    voting_key: str


class Drep(BaseModel):
    """Represents a DREP from the GVC API."""

    id: int
    attributes: DrepAttributes


class Client:
    """GVC API client."""

    def __init__(self, api_url: str):
        """Initialize the client."""
        self.inner = utils.HttpClient(api_url)

    async def close(self):
        await self.inner.close()

    async def dreps(self) -> List[Drep]:
        """Get all DREPs."""
        res = await self.inner.json_get("/dreps")
        if not isinstance(res, dict):
            raise utils.BadResponse()

        return [Drep.model_validate(e) for e in res["data"]]
