import json
import asyncpg
import dataclasses
from typing import Any, List, Optional, TypeVar

from .models import Model


M = TypeVar("M", bound=Model)


class VoteOptionsNotFound(Exception):
    ...


class Connection:
    def __init__(self, conn: asyncpg.Connection):
        self.conn = conn

    async def close(self):
        await self.conn.close()

    def transaction(self):
        return self.conn.transaction()

    async def insert_many(
        self,
        models: List[M],
        returning: Optional[str] = None
    ) -> List[Any]:
        """
        Batch inserts all models.
        """

        if len(models) == 0:
            return []

        # Extract field names and values for each field in each model
        cols = [field.name for field in dataclasses.fields(models[0])]
        vals = [[getattr(m, f) for f in cols] for m in models]

        # Creates a list for the placeholders for value params, e.g. ["($1, $2, $3)", "($4, $5, $6)"]
        val_nums = []
        for i in range(0, len(vals)):
            nums = range(i*len(cols)+1, (i+1)*len(cols)+1)
            nums_str = ",".join(map(lambda x: f"${x}", nums))
            val_nums.append(f"({nums_str})")

        val_nums_str = ",".join(val_nums)
        cols_str = ','.join(cols)

        flat_vals = [v for vs in vals for v in vs]

        stmt_template = f"INSERT INTO {models[0].table()} ({cols_str}) VALUES {val_nums_str}"

        if returning is None:
            await self.conn.execute(stmt_template, *flat_vals)
            return []
        else:
            stmt_template += f" RETURNING {returning}"
            ret = await self.conn.fetch(stmt_template, *flat_vals)
            return [record[returning] for record in ret]

    async def insert(self, model: Model, returning: Optional[str] = None) -> Any:
        ret = await self.insert_many([model], returning)
        if len(ret) > 0:
            return ret[0]
        return None

    async def election_exists(self, id: int) -> bool:
        """
        Checks whether a election exists with the given id.
        """

        row = await self.conn.fetchrow("SELECT row_id FROM election WHERE row_id = $1", id)
        return row is not None

    async def get_vote_options_id(self, challenge: str) -> int:
        """
        Gets the id of the vote option matching the given challenge.
        """

        row = await self.conn.fetchrow("SELECT id FROM vote_options WHERE challenge = $1", challenge)
        if row is None:
            raise VoteOptionsNotFound()
        return row["id"]


async def connect(url: str) -> Connection:
    conn = await asyncpg.connect(url)
    await conn.set_type_codec("jsonb", encoder=json.dumps, decoder=json.loads, schema="pg_catalog")

    return Connection(conn)
