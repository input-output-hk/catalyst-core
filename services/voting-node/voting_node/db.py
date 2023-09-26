"""Provides an instance connected to the EventDB, as well as queries used by the running node schedule."""
import datetime
import os

import asyncpg
from asyncpg import Connection
from loguru import logger
from pydantic import BaseModel

from .envvar import SECRET_SECRET
from .models import Contribution, Event, HostInfo, LeaderHostInfo, Objective, Proposal, Snapshot, VotePlan, Voter, VotingGroup
from .utils import LEADER_REGEX, decrypt_secret, encrypt_secret, get_hostname


class EventDb(BaseModel):
    """Convenient abstraction to call the EventDB within the voting node service.

    Each method that queries the EventDB opens and closes its own connection.
    """

    db_url: str = ""
    connection: Connection | None = None

    class Config:
        """Pydantic model configuration parameters."""

        arbitrary_types_allowed = True

    async def connect(self):
        """Create a connection to the DB."""
        conn = await asyncpg.connect(self.db_url)
        if conn is None:
            raise Exception("failed to connect to the database")
        self.connection = conn
        return conn

    async def close(self):
        """Close a connection to the DB."""
        if self.connection is not None:
            await self.connection.close()

    async def fetch_upcoming_event(self) -> Event:
        """Look in EventDB for the next event that will start."""
        # first, check if there is an event that has not finished
        conn = await self.connect()
        now = datetime.datetime.utcnow()
        query = """
        SELECT
            *
        FROM
            event
        WHERE
            start_time <= $1 AND $1 < end_time
        ORDER BY
            start_time ASC
        LIMIT 1"""
        result = await conn.fetchrow(query, now)
        await conn.close()

        if result is None:
            raise Exception("failed to fetch event from DB")

        event = Event(**dict(result))
        logger.debug(f"fetched upcoming event: {event.name}")
        return event

    async def fetch_leader_host_info(self, event_row_id: int) -> HostInfo:
        """Return HostInfo for leaders, sorted by hostname."""
        conn = await self.connect()
        conds = "hostname = $1 AND event = $2"
        query = f"SELECT * FROM voting_node WHERE {conds}"
        result = await conn.fetchrow(query, get_hostname(), event_row_id)
        await conn.close()

        match result:
            case None:
                raise Exception("failed to fetch leader node info from DB")
            case record:
                # fetch secret from envvar, fail if not present
                encrypt_pass = os.environ[SECRET_SECRET]
                seckey = decrypt_secret(record["seckey"], encrypt_pass)
                netkey = decrypt_secret(record["netkey"], encrypt_pass)
                host_info = HostInfo(
                    hostname=record["hostname"],
                    event=record["event"],
                    seckey=seckey,
                    pubkey=record["pubkey"],
                    netkey=netkey,
                )
                return host_info

    async def insert_leader_host_info(self, host_info: HostInfo):
        """Insert the hostname row into the voting_node table."""
        conn = await self.connect()
        fields = "hostname, event, seckey, pubkey, netkey"
        values = "$1, $2, $3, $4, $5"
        query = f"INSERT INTO voting_node({fields}) VALUES({values}) RETURNING *"
        h = host_info
        # fetch secret from envvar, fail if not present
        encrypt_pass = os.environ[SECRET_SECRET]

        enc_sk = encrypt_secret(h.seckey, encrypt_pass)
        enc_nk = encrypt_secret(h.netkey, encrypt_pass)
        result = await conn.execute(
            query,
            h.hostname,
            h.event,
            enc_sk,
            h.pubkey,
            enc_nk,
        )
        await conn.close()

        if result is None:
            raise Exception(f"failed to insert '{h.hostname}' info to DB")
        logger.debug(f"{h.hostname} info added: {result}")

    async def fetch_sorted_leaders_host_info(self, event_row_id: int) -> list[LeaderHostInfo]:
        """Return a list of leader host information.

        Fetch host information for leader nodes.
        Raises exceptions if the DB fails to return a list of records, or if the list is empty.
        """
        conn = await self.connect()
        query = f"""
        SELECT (hostname, pubkey)
        FROM voting_node
        WHERE hostname ~ '{LEADER_REGEX}' AND event = $1
        ORDER BY hostname ASC"""
        result = await conn.fetch(query, event_row_id)
        await conn.close()

        match result:
            case None:
                raise Exception("DB error fetching leaders host info")
            case []:
                raise Exception("no leader host info found in DB")
            case [*leaders]:

                def extract_leader_info(leader):
                    row = leader["row"]
                    host_info = LeaderHostInfo(hostname=row[0], consensus_leader_id=row[1], role=None)
                    logger.debug(f"{host_info.hostname}")
                    return host_info

                logger.debug(f"found {len(leaders)} leaders")
                extracted_leaders = [extract_leader_info(leader) for leader in leaders]
                return extracted_leaders

    async def fetch_proposals(self, objective_id: int) -> list[Proposal]:
        """Return a list of proposals ."""
        conn = await self.connect()
        query = "SELECT * FROM proposal WHERE objective = $1"
        result = await conn.fetch(query, objective_id)
        await conn.close()

        if result is None:
            raise Exception("proposals DB error")
        logger.debug("proposals retrieved from DB", objective=objective_id)
        match result:
            case None:
                raise Exception("DB error fetching proposals")
            case []:
                raise Exception("no proposals found in DB")
            case [*proposals]:
                logger.debug(f"proposals retrieved from DB: {len(proposals)}")
                return [Proposal(**dict(r)) for r in proposals]

    async def check_if_snapshot_is_final(self, event_id: int) -> bool:
        """Query if the snapshot is finalized."""
        conn = await self.connect()
        query = "SELECT final FROM snapshot WHERE event = $1"
        result = await conn.fetchrow(query, event_id)
        await conn.close()

        match result:
            case None:
                raise Exception("snapshot DB error")
            case record:
                final = record.get("final")
                if final is None:
                    raise Exception("expected the snapshot to have the final field set")

                final_flag = bool(final)
                logger.debug(f"snapshot finalized? {final_flag}")
                return final_flag

    async def fetch_snapshot(self, event_id: int) -> Snapshot:
        """Fetch the snapshot row for the event_id."""
        conn = await self.connect()
        # fetch the voters
        columns = "(row_id, event, as_at, last_updated, dbsync_snapshot_data"
        columns += ", drep_data, catalyst_snapshot_data, final)"
        query = f"SELECT {columns} FROM snapshot WHERE event = $1"
        result = await conn.fetchrow(query, event_id)
        await conn.close()

        if result is None:
            raise Exception("snapshot DB error")
        logger.debug("snapshot retrieved from DB")
        match result:
            case None:
                raise Exception("DB error fetching snapshot")
            case snpsht:
                snapshot = Snapshot(*snpsht["row"])
                logger.debug("snapshot retrieved from DB")
                return snapshot

    async def fetch_voting_groups(self) -> list[VotingGroup]:
        """Fetch the voters registered for the event_id."""
        conn = await self.connect()
        query = "SELECT * FROM voting_group"
        result = await conn.fetch(query)
        await conn.close()

        match result:
            case None:
                raise Exception("DB error fetching voting groups")
            case [*items]:
                logger.debug("voting groups retrieved from DB: {group_count}", group_count=len(items))
                return [VotingGroup(**dict(r)) for r in items]

    async def fetch_voters(self, event_id: int) -> list[Voter]:
        """Fetch the voters registered for the event_id."""
        conn = await self.connect()
        query = """
        SELECT * FROM voter WHERE snapshot_id IN (SELECT row_id FROM snapshot WHERE event = $1)
        """
        result = await conn.fetch(query, event_id)
        await conn.close()

        match result:
            case None:
                raise Exception("DB error fetching voters")
            case [*voters]:
                logger.debug(f"voters retrieved from DB: {len(voters)}")
                return [Voter(**dict(r)) for r in voters]

    async def fetch_contributions(self, event_id: int) -> list[Contribution]:
        """Fetch the contributions registered for the event_id."""
        conn = await self.connect()
        query = """
        SELECT * FROM contribution WHERE snapshot_id IN (SELECT row_id FROM snapshot WHERE event = $1)
        """
        result = await conn.fetch(query, event_id)
        await conn.close()

        match result:
            case None:
                raise Exception("DB error fetching contributions")
            case [*contributions]:
                logger.debug(f"contributions retrieved from DB: {len(contributions)}")
                return [Contribution(**dict(r)) for r in contributions]

    async def fetch_objectives(self, event_id: int) -> list[Objective]:
        """Fetch the objectives for the event_id."""
        conn = await self.connect()
        query = "SELECT * FROM objective WHERE event = $1"
        result = await conn.fetch(query, event_id)
        await conn.close()

        match result:
            case None:
                raise Exception("DB error fetching voteplans")
            case [*items]:
                logger.debug("voteplans retrieved from DB: {objective_count}", objective_count=len(items))
                return [Objective(**dict(r)) for r in items]

    async def fetch_voteplans(self, event_id: int) -> list[VotePlan]:
        """Fetch the voteplans for the event_id."""
        conn = await self.connect()
        query = "SELECT * FROM voteplan WHERE objective_id IN (SELECT row_id FROM objective WHERE event = $1) ORDER BY id ASC"
        result = await conn.fetch(query, event_id)
        await conn.close()

        match result:
            case None:
                raise Exception("DB error fetching voteplans")
            case [*voteplans]:
                logger.debug(f"voteplans retrieved from DB: {len(voteplans)}")
                return [VotePlan(**dict(r)) for r in voteplans]

    async def insert_block0_info(self, event_row_id: int, block0_bytes: bytes, block0_hash: str):
        """Update the event with the given block0 bytes and hash."""
        conn = await self.connect()
        columns = "block0 = $1, block0_hash = $2"
        condition = "row_id = $3"
        returning = "name"
        query = f"UPDATE event SET {columns} WHERE {condition} RETURNING {returning}"
        try:
            result = await conn.execute(query, block0_bytes, block0_hash, event_row_id)

            if result is None:
                raise Exception("failed to insert block0 info from DB")
            logger.debug(f"block0 info added to event: {result}")
        except Exception as e:
            raise Exception(f"inserting block0 info went wrong: {e}") from e
        finally:
            await conn.close()
