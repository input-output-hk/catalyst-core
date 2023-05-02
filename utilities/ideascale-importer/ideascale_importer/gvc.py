"""GVC API module."""

from pydantic.dataclasses import dataclass
import pydantic.tools
from typing import List

from ideascale_importer import utils


@dataclass
class DrepAttributes:
    """Represents DREP attributes from the GVC API."""

    ...


@dataclass
class Drep:
    """Represents a DREP from the GVC API."""

    id: int
    attributes: DrepAttributes


class Client:
    """GVC API client."""

    def __init__(self, api_url: str):
        """Initialize the client."""
        self.inner = utils.JsonHttpClient(api_url)

    async def dreps(self) -> List[Drep]:
        """Get all DREPs."""
        res = await self.inner.get("/api/dreps")
        return [pydantic.tools.parse_obj_as(Drep, e) for e in res]
