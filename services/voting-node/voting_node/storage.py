"""Storage types for the voting node."""
from pathlib import Path

from pydantic import BaseModel

from .db import EventDb
from .logs import getLogger
from .models import Committee, YamlFile, YamlType

# gets voting node logger
logger = getLogger()


class SecretDBStorage(BaseModel):
    """DB secret storage for secrets."""

    db: EventDb

    class Config:
        arbitrary_types_allowed = True

    async def get_committee(self, event_id: int) -> Committee:
        """Fetch the voteplans for the event_id."""
        fields = "row_id, event as event_id, committee_id, member_crs as crs, election_key"
        query = f"SELECT {fields} FROM tally_committee WHERE event = $1"
        result = await self.db.conn().fetchrow(query, event_id)
        match result:
            case record if record is not None:
                return Committee(**dict(record))
            case _:
                raise Exception("expected tally committee in DB")

    async def save_committee(self, committee: Committee):
        data = committee.dict()
        tuple(data.keys())
        tuple(data.values())


class SecretFileStorage(BaseModel):
    """File storage for secrets."""

    path: Path

    async def get_committee(self) -> Committee:
        file = self.path.joinpath("committee.yaml")
        committee = await Committee.read_file(file)
        return committee

    async def save_committee(self, committee: Committee):
        file = self.path.joinpath("committee.yaml")
        await YamlFile(yaml_type=YamlType(content=committee.dict()), path=file).save()
