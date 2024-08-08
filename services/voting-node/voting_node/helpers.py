import asyncio
from datetime import datetime, timedelta
from typing import Final

import asyncpg
from loguru import logger

SLOT_DURATION: Final = 4
SLOTS_PER_EPOCH: Final = 900
DEFAULT_EVENT_START_DATE: Final = datetime.utcnow() + timedelta(seconds=30)


def slotdelta(epochs: int = 0, slots: int = 0):
    slots_in_secs = (epochs * SLOTS_PER_EPOCH + slots) * SLOT_DURATION
    return timedelta(seconds=slots_in_secs)


async def add_default_event(
    db_url: str = "postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev",
    reference_date: datetime = DEFAULT_EVENT_START_DATE,
):
    """Async function that creates a new event in the database with the provided parameters.

    Args:
    ----
        reference_date (datetime): The reference date to calculate the event's timing based on, defaults to datetime.utcnow().

    Returns:
    -------
        None.
    """
    # Execute a statement to create a new event.
    conn = await asyncpg.connect(db_url)
    if conn is None:
        raise Exception("no db connection found for")

    block0_date = reference_date
    # dates
    start_time = block0_date

    # moment that registrations from Cardano main net are frozen
    registration_snapshot_time = block0_date + slotdelta(slots=2)
    # the moment that registrations are considered to be stable
    snapshot_start = registration_snapshot_time + timedelta(days=14)

    voting_start = snapshot_start + slotdelta(slots=15)
    voting_end = voting_start + timedelta(days=14)
    tallying_end = voting_end + timedelta(days=1)

    end_time = tallying_end + slotdelta(slots=5)  # finish event 20 secs after tallying_end

    voting_power_threshold = 450
    # Integer up to 100
    max_voting_power_pct = 100

    insight_sharing_start = block0_date + timedelta(minutes=4)
    proposal_submission_start = block0_date + timedelta(minutes=5)
    refine_proposals_start = block0_date + timedelta(minutes=6)
    finalize_proposals_start = block0_date + timedelta(minutes=7)
    proposal_assessment_start = block0_date + timedelta(minutes=8)
    assessment_qa_start = block0_date + timedelta(minutes=9)

    committee_size = 5
    committee_threshold = 3

    query = """
        INSERT INTO
        event(
            row_id,
            name,
            description,
            registration_snapshot_time,
            voting_power_threshold,
            max_voting_power_pct,
            start_time,
            end_time,
            insight_sharing_start,
            proposal_submission_start,
            refine_proposals_start,
            finalize_proposals_start,
            proposal_assessment_start,
            assessment_qa_start,
            snapshot_start,
            voting_start,
            voting_end,
            tallying_end,
            committee_size,
            committee_threshold
            )
        VALUES (
            (SELECT COALESCE(MAX(row_id), 0) + 1 FROM event),
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        RETURNING *
    """
    await conn.execute(
        query,
        f"Fund TEST {start_time}",
        "Description for the Fund.",
        registration_snapshot_time,
        voting_power_threshold,
        max_voting_power_pct,
        start_time,
        end_time,
        insight_sharing_start,
        proposal_submission_start,
        refine_proposals_start,
        finalize_proposals_start,
        proposal_assessment_start,
        assessment_qa_start,
        snapshot_start,
        voting_start,
        voting_end,
        tallying_end,
        committee_size,
        committee_threshold,
    )

    logger.debug("inserted upcoming event")

    # Close the connection.
    await conn.close()


async def delete_table(table_name: str, db_url: str = "postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"):
    conn = await asyncpg.connect(db_url)
    if conn is None:
        raise Exception("no db connection found for")

    result = await conn.execute(
        f"DELETE FROM  {table_name}",
    )

    logger.debug(f"deleted table rows from '{table_name}': {result}")
    await conn.close()


if __name__ == "__main__":
    """Reset the database when called from the command line."""
    asyncio.run(add_default_event())
