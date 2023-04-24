"""Database module.

This module contains functions for interacting with the database.
"""

import json
import asyncpg
import dataclasses
from typing import Any, List, Optional, TypeVar

from .models import Model, Contribution, Voter


M = TypeVar("M", bound=Model)


async def insert_many(conn: asyncpg.Connection, models: List[M], returning: Optional[str] = None) -> List[Any]:
    """Batch inserts all models."""
    if len(models) == 0:
        return []

    # Extract field names and values for each field in each model
    cols = [field.name for field in dataclasses.fields(models[0])]
    vals = [[getattr(m, f) for f in cols] for m in models]

    # Creates a list for the placeholders for value params, e.g. ["($1, $2, $3)", "($4, $5, $6)"]
    val_nums = []
    for i in range(0, len(vals)):
        nums = range(i * len(cols) + 1, (i + 1) * len(cols) + 1)
        nums_str = ",".join(map(lambda x: f"${x}", nums))
        val_nums.append(f"({nums_str})")

    val_nums_str = ",".join(val_nums)
    cols_str = ",".join(cols)

    flat_vals = [v for vs in vals for v in vs]

    stmt_template = f"INSERT INTO {models[0].table()} ({cols_str}) VALUES {val_nums_str}"

    if returning is None:
        await conn.execute(stmt_template, *flat_vals)
        return []
    else:
        stmt_template += f" RETURNING {returning}"
        ret = await conn.fetch(stmt_template, *flat_vals)
        return [record[returning] for record in ret]


async def insert(conn: asyncpg.Connection, model: Model, returning: Optional[str] = None) -> Any:
    """Insert a single model.

    If returning is not None, returns the value of that column.
    """
    ret = await insert_many(conn, [model], returning)
    if len(ret) > 0:
        return ret[0]
    return None


async def upsert_many(
    conn: asyncpg.Connection,
    models: List[M],
    conflict_cols: List[str],
    exclude_update_cols: List[str] = [],
    returning: Optional[str] = None,
) -> List[Any]:
    """Batch upserts models of the same type.

    conflict_cols is a list of columns that are used to determine whether a row should be updated or inserted.
    If returning is not None, returns the values of that column.
    """
    if len(models) == 0:
        return []

    # Extract field names and values for each field in each model
    cols = [field.name for field in dataclasses.fields(models[0])]
    vals = [[getattr(m, f) for f in cols] for m in models]

    # Creates a list for the placeholders for value params, e.g. ["($1, $2, $3)", "($4, $5, $6)"]
    val_nums = []
    for i in range(0, len(vals)):
        nums = range(i * len(cols) + 1, (i + 1) * len(cols) + 1)
        nums_str = ",".join(map(lambda x: f"${x}", nums))
        val_nums.append(f"({nums_str})")

    val_nums_str = ",".join(val_nums)
    cols_str = ",".join(cols)

    flat_vals = [v for vs in vals for v in vs]

    conflict_cols_str = ",".join(conflict_cols)
    do_update_set_str = ",".join([f"{col} = EXCLUDED.{col}" for col in cols if col not in exclude_update_cols])

    stmt_template = f"""
        INSERT INTO {models[0].table()} ({cols_str}) VALUES {val_nums_str}
        ON CONFLICT ({conflict_cols_str})
        DO UPDATE
        SET {do_update_set_str}
    """.strip()

    if returning is None:
        await conn.execute(stmt_template, *flat_vals)
        return []
    else:
        stmt_template += f" RETURNING {returning}"
        ret = await conn.fetch(stmt_template, *flat_vals)
        return [record[returning] for record in ret]


async def upsert(
    conn: asyncpg.Connection,
    model: Model,
    conflict_cols: List[str],
    exclude_update_cols: List[str] = [],
    returning: Optional[str] = None,
):
    """Upsert a single model.

    conflict_cols is a list of columns that are used to determine whether a row should be updated or inserted.
    If returning is not None, returns the value of that column.
    """
    ret = await upsert_many(conn, [model], conflict_cols, exclude_update_cols, returning)
    if len(ret) > 0:
        return ret[0]
    return None


async def delete_snapshot_data(conn, snapshot_id: int):
    """Delete all data associated with the given snapshot."""
    await conn.execute(f"DELETE FROM {Contribution.table()} WHERE snapshot_id = $1", snapshot_id)
    await conn.execute(f"DELETE FROM {Voter.table()} WHERE snapshot_id = $1", snapshot_id)


async def event_exists(conn: asyncpg.Connection, id: int) -> bool:
    """Check whether a event exists with the given id."""
    row = await conn.fetchrow("SELECT row_id FROM event WHERE row_id = $1", id)
    return row is not None


class VoteOptionsNotFound(Exception):
    """Raised when a vote option is not found."""

    ...


async def get_vote_options_id(conn: asyncpg.Connection, objective: str) -> int:
    """Get the id of the vote option matching the given objective."""
    row = await conn.fetchrow("SELECT id FROM vote_options WHERE objective = $1", objective)
    if row is None:
        raise VoteOptionsNotFound()
    return row["id"]


async def connect(url: str) -> asyncpg.Connection:
    """Return a connection to the database.

    This also sets the jsonb codec to use the json module.
    """
    conn = await asyncpg.connect(url)
    await conn.set_type_codec("jsonb", encoder=json.dumps, decoder=json.loads, schema="pg_catalog")
    return conn
