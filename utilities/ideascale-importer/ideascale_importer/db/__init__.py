"""Database module.

This module contains functions for interacting with the database.
"""

import json
import asyncpg
import dataclasses
from typing import Any, List, Dict, TypeVar

from .models import Model, Contribution, Voter


PG_MAX_QUERY_PARAMS = 32767
M = TypeVar("M", bound=Model)


async def insert_many(conn: asyncpg.Connection, models: List[M]) -> List[M]:
    """Batch inserts all models."""
    if len(models) == 0:
        return []

    # Extract field names and values for each field in each model
    cols = [field.name for field in dataclasses.fields(models[0])]
    insert_cols = [col for col in cols if col not in models[0].exclude_from_insert]
    cols_str = ",".join(cols)
    insert_cols_str = ",".join(insert_cols)
    vals = [[getattr(m, f) for f in cols] for m in models]

    # Creates a list for the placeholders for value params, e.g. ["($1, $2, $3)", "($4, $5, $6)"]
    val_nums = []
    cur_vals = []
    k = 1
    results = []
    for i in range(len(vals)):
        nums = range(k, k + len(insert_cols))
        k += len(insert_cols)

        nums_str = ",".join(map(lambda x: f"${x}", nums))

        cur_vals.extend(vals[i])
        val_nums.append(f"({nums_str})")

        if len(cur_vals) + len(insert_cols) >= PG_MAX_QUERY_PARAMS or i == (len(vals) - 1):
            val_nums_str = ",".join(val_nums)
            stmt_template = f"INSERT INTO {models[0].table()} ({insert_cols_str}) VALUES {val_nums_str} RETURNING {cols_str}"
            result = await conn.fetch(stmt_template, *cur_vals)
            results.extend([models[0].__class__(**record) for record in result])

            cur_vals = []
            val_nums = []
            k = 1

    return results


async def insert(conn: asyncpg.Connection, model: Model) -> Any:
    """Insert a single model.

    If returning is not None, returns the value of that column.
    """
    ret = await insert_many(conn, [model])
    if len(ret) > 0:
        return ret[0]
    return None

async def select(conn: asyncpg.Connection, model: Model, cond: Dict[str, str] = {}) -> List[Any]:
    """Select a single model."""

    # Extract field names and values for each field in each model
    cols = [field.name for field in dataclasses.fields(model)]

    cols_str = ",".join(cols)
    cond_str = " ".join([f"{col} {cond}" for col, cond in cond.items()])

    stmt_template = f"""
        SELECT {cols_str}
        FROM {model.table()}
        {f' WHERE {cond_str}' if cond_str else ' '}
    """.strip()  

    result = await conn.fetch(stmt_template)

    return [model.__class__(**record) for record in result]


async def upsert_many(
    conn: asyncpg.Connection,
    models: List[M],
    conflict_cols: List[str],
    exclude_update_cols: List[str] = [],
    pre_update_cols: Dict[str, str] = {},
    pre_update_cond: Dict[str, str] = {},
) -> List[Any]:
    """Batch upserts models of the same type.

    conflict_cols is a list of columns that are used to determine whether a row should be updated or inserted.
    If returning is not None, returns the values of that column.
    """
    if len(models) == 0:
        return []

    # Extract field names and values for each field in each model
    cols = [field.name for field in dataclasses.fields(models[0])]
    insert_cols = [col for col in cols if col not in models[0].exclude_from_insert]
    vals = [[getattr(m, f) for f in insert_cols] for m in models]

    # Creates a list for the placeholders for value params, e.g. ["($1, $2, $3)", "($4, $5, $6)"]
    val_nums = []
    for i in range(0, len(vals)):
        nums = range(i * len(insert_cols) + 1, (i + 1) * len(insert_cols) + 1)
        nums_str = ",".join(map(lambda x: f"${x}", nums))
        val_nums.append(f"({nums_str})")

    val_nums_str = ",".join(val_nums)
    cols_str = ",".join(cols)
    insert_cols_str = ",".join(insert_cols)

    flat_vals = [v for vs in vals for v in vs]

    conflict_cols_str = ",".join(conflict_cols)
    do_update_set_str = ",".join([f"{col} = EXCLUDED.{col}" for col in insert_cols if col not in exclude_update_cols])
    pre_update_set_str = ",".join([f"{col} = {val}" for col, val in pre_update_cols.items()])
    pre_update_cond_str = " ".join([f"{col} {cond}" for col, cond in pre_update_cond.items()])

    pre_update_template = f"""
        WITH updated AS ({ f"UPDATE {models[0].table()} SET {pre_update_set_str} {f' WHERE {pre_update_cond_str}' if pre_update_cond_str else ' '}" })
        """.strip() if pre_update_set_str else " "     

    stmt_template = f"""
        {pre_update_template}

        INSERT INTO {models[0].table()} ({insert_cols_str}) VALUES {val_nums_str}
        ON CONFLICT ({conflict_cols_str})
        DO UPDATE
        SET {do_update_set_str}
        RETURNING {cols_str}
    """.strip()

    result = await conn.fetch(stmt_template, *flat_vals)

    return [models[0].__class__(**record) for record in result]


async def upsert(
    conn: asyncpg.Connection,
    model: Model,
    conflict_cols: List[str],
    exclude_update_cols: List[str] = [],
    pre_update_cols: Dict[str, str] = {},
):
    """Upsert a single model.

    conflict_cols is a list of columns that are used to determine whether a row should be updated or inserted.
    If returning is not None, returns the value of that column.
    """
    ret = await upsert_many(conn, [model], conflict_cols, exclude_update_cols, pre_update_cols)
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


async def get_vote_options_id(conn: asyncpg.Connection, objective: List[str]) -> int:
    """Get the id of the vote option matching the given objective."""
    row = await conn.fetchrow("SELECT id FROM vote_options WHERE objective = $1", objective)
    if row is None:
        raise VoteOptionsNotFound()
    return row["id"]


async def connect(url: str, *args, **kwargs) -> asyncpg.Connection:
    """Return a connection to the database.

    This also sets the jsonb codec to use the json module.
    """
    try:
        conn = await asyncpg.connect(dsn=url, *args, **kwargs)
    except Exception as _:
        raise Exception("Database connection failed")

    try:
        await conn.set_type_codec("jsonb", encoder=json.dumps, decoder=json.loads, schema="pg_catalog")
    except Exception as _:
        raise Exception("Failed to set jsonb codec")

    return conn
