"""Provides an instance connected to the EventDB, as well as queries used by the running node schedule."""
import datetime

import asyncpg
from asyncpg import Connection
from pydantic import BaseModel

from .logs import getLogger
from .models import Event, HostInfo, LeaderHostInfo, Proposal, Snapshot, VotePlan
from .utils import LEADER_REGEX, get_hostname

# gets voting node logger
logger = getLogger()


class EventDb(BaseModel):
    """Convenient abstraction to call the EventDB within the voting node service."""

    connection: Connection | None = None
    db_url: str

    class Config:
        """Pydantic model configuration parameters."""
        arbitrary_types_allowed = True


    async def connect(self):
        """Create a connection to the DB."""
        conn = await asyncpg.connect(self.db_url)
        if conn is None:
            raise Exception("failed to connect to the database")
        self.connection = conn

    def conn(self) -> Connection:
        """Return the connection to the DB.

        Raise exception if connection is None.
        """
        if self.connection is None:
            raise Exception("no connection to EventDB found")
        return self.connection

    async def close(self):
        """Close a connection to the DB."""
        if self.connection is not None:
            await self.connection.close()

    async def fetch_current_event(self) -> Event:
        """Look in EventDB for the event that will start voting."""
        # first, check if there is an event that has not finished
        now = datetime.datetime.utcnow()
        filter_by = "(voting_end > $1 or voting_end = $2) and voting_start < $1"
        sort_by = "voting_start ASC"
        query = f"SELECT * FROM event WHERE {filter_by} ORDER BY {sort_by} LIMIT 1"
        result = await self.conn().fetchrow(query, now, None)
        if result is not None:
            logger.debug(f"fetched ongoing event: {result}")
            return Event(**dict(result))

        filter_by = "voting_start > $1"
        query = f"SELECT * FROM event WHERE {filter_by} ORDER BY {sort_by} LIMIT 1"
        result = await self.conn().fetchrow(query, now)
        if result is None:
            raise Exception("failed to fetch event from DB")
        logger.debug(f"fetched upcoming event: {result}")
        return Event(**dict(result))

    async def fetch_leader_host_info(self, event_row_id: int) -> HostInfo:
        """Return HostInfo for leaders, sorted by hostname."""
        filter_by = "hostname = $1 AND event = $2"
        query = f"SELECT * FROM voting_node WHERE {filter_by}"
        result = await self.conn().fetchrow(query, get_hostname(), event_row_id)
        if result is None:
            raise Exception("failed to fetch leader node info from DB")
        host_info = HostInfo(**dict(result))
        return host_info

    async def insert_leader_host_info(self, host_info: HostInfo):
        """Insert the hostname row into the voting_node table."""
        fields = "hostname, event, seckey, pubkey, netkey"
        values = "$1, $2, $3, $4, $5"
        query = f"INSERT INTO voting_node({fields}) VALUES({values}) RETURNING *"
        h = host_info
        result = await self.conn().execute(
            query,
            h.hostname,
            h.event,
            h.seckey,
            h.pubkey,
            h.netkey,
        )
        if result is None:
            raise Exception(f"failed to insert '{h.hostname}' info to DB")
        logger.debug(f"{h.hostname} info added: {result}")

    async def fetch_sorted_leaders_host_info(self) -> list[LeaderHostInfo]:
        """Return a list of leader host information.

        Fetch host information for leader nodes.
        Raises exceptions if the DB fails to return a list of records, or if the list is empty.
        """
        where = f"WHERE hostname ~ '{LEADER_REGEX}'"
        order_by = "ORDER BY hostname ASC"
        query = f"SELECT (hostname, pubkey) FROM voting_node {where} {order_by}"
        result = await self.conn().fetch(query)
        match result:
            case None:
                raise Exception("DB error fetching leaders host info")
            case []:
                raise Exception("no leader host info found in DB")
            case [*leaders]:
                logger.debug(f"found leaders: {leaders}")

                def extract_leader_info(leader):
                    host_info = LeaderHostInfo(*leader["row"])
                    logger.debug(f"{host_info}")
                    return host_info

                return list(map(extract_leader_info, leaders))

    async def fetch_proposals(self) -> list[Proposal]:
        """Return a list of proposals ."""
        query = "SELECT * FROM proposal ORDER BY id ASC"
        result = await self.conn().fetch(query)
        if result is None:
            raise Exception("proposals DB error")
        logger.debug(f"proposals retrieved from DB: {len(result)}")
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
        query = "SELECT final FROM snapshot WHERE event = $1"
        result = await self.conn().fetchrow(query, event_id)
        match result:
            case None:
                raise Exception("snapshot DB error")
            case record:
                final = record.get("final")
                logger.debug(f"snapshot finalized? {final}")
                return final

    async def fetch_snapshot(self, event_id: int) -> Snapshot:
        """Fetch the snapshot row for the event_id."""
        # fetch the voters
        columns = "(row_id, event, as_at, last_updated, dbsync_snapshot_data"
        columns += ", drep_data, catalyst_snapshot_data, final)"
        query = f"SELECT {columns} FROM snapshot WHERE event = $1"
        result = await self.conn().fetchrow(query, event_id)
        if result is None:
            raise Exception("snapshot DB error")
        logger.debug(f"snapshot retrieved from DB: {result}")
        match result:
            case None:
                raise Exception("DB error fetching snapshot")
            case snpsht:
                snapshot = Snapshot(*snpsht["row"])
                logger.debug(f"snapshot retrieved from DB: {snapshot}")
                return snapshot

    async def fetch_voteplans(self, event_id: int) -> list[VotePlan]:
        """Fetch the voteplans for the event_id."""
        query = "SELECT * FROM voteplan WHERE event_id = $1 ORDER BY id ASC"
        result = await self.conn().fetch(query, event_id)
        if result is None:
            raise Exception("voteplan DB error")
        logger.debug(f"voteplans retrieved from DB: {len(result)}")
        match result:
            case None:
                raise Exception("DB error fetching voteplans")
            case []:
                raise Exception("no voteplans found in DB")
            case [*voteplans]:
                logger.debug(f"voteplans retrieved from DB: {len(voteplans)}")
                return [VotePlan(**dict(r)) for r in voteplans]

    async def insert_block0_info(self, event_row_id: int, block0_bytes: bytes, block0_hash: str):
        """Update the event with the given block0 bytes and hash."""
        columns = "block0 = $1, block0_hash = $2"
        condition = "row_id = $3"
        returning = "name"
        query = f"UPDATE event SET {columns} WHERE {condition} RETURNING {returning}"
        try:
            result = await self.conn().execute(query, block0_bytes, block0_hash, event_row_id)
            if result is None:
                raise Exception("failed to insert block0 info from DB")
            logger.debug(f"block0 info added to event: {result}")
        except Exception as e:
            raise Exception(f"inserting block0 info went wrong: {e}") from e
