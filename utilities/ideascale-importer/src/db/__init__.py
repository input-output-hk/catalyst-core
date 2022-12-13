import asyncpg
import dataclasses
from typing import Any, List, Optional, Tuple, TypeVar

from .models import Model


M = TypeVar("M", bound=Model)


def items(models: List[M]) -> Tuple[List[str], List[List[Any]]]:
    if len(models) == 0:
        raise Exception("No models")

    field_names = [field.name for field in dataclasses.fields(models[0])]
    return (field_names, [[getattr(m, f) for f in field_names] for m in models])


async def insert_many(
    conn: asyncpg.Connection,
    models: List[M],
    returning: Optional[str] = None
) -> List[Any]:
    if len(models) == 0:
        return []

    cols, vals = items(models)

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
        await conn.execute(stmt_template, *flat_vals)
        return []
    else:
        stmt_template += f" RETURNING {returning}"
        ret = await conn.fetch(stmt_template, *flat_vals)
        return [record[returning] for record in ret]


async def election_exists(conn: asyncpg.Connection, id: int) -> bool:
    row = await conn.fetchrow("SELECT row_id FROM election WHERE row_id = $1", id)
    return row is not None
