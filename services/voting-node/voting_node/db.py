import asyncpg
import datetime
from typing import Any, List

from .logs import getLogger
from .models import Event, NodeInfo, PeerNode, Proposal
from .utils import get_hostname

# gets voting node logger
logger = getLogger()


class EventDb(object):
    conn: Any = None
    db_url: str

    def __init__(self, db_url: str) -> None:
        self.db_url = db_url

    async def connect(self):
        conn = await asyncpg.connect(self.db_url)
        if conn is None:
            raise Exception("failed to connect to the database")
        self.conn = conn

    async def close(self):
        if self.conn is not None:
            await self.conn.close()

    async def fetch_upcoming_event(self) -> Event:
        filter_by = "voting_start > $1"
        sort_by = "voting_start ASC"
        query = f"SELECT * FROM event WHERE {filter_by} ORDER BY {sort_by} LIMIT 1"
        now = datetime.datetime.utcnow()
        result = await self.conn.fetchrow(query, now)
        if result is None:
            raise Exception("failed to fetch event from db")
        return Event(**dict(result))

    async def fetch_leader_node_info(self, event_row_id: int) -> NodeInfo:
        filter_by = "hostname = $1 AND event = $2"
        query = f"SELECT * FROM voting_node WHERE {filter_by}"
        result = await self.conn.fetchrow(query, get_hostname(), event_row_id)
        if result is None:
            raise Exception("failed to fetch leader node info from db")
        node_info = NodeInfo(**dict(result))
        return node_info

    async def insert_leader_node_info(self, node_info: NodeInfo):
        # insert the hostname row into the voting_node table
        fields = "hostname, event, seckey, pubkey, netkey"
        values = "$1, $2, $3, $4, $5"
        query = f"INSERT INTO voting_node({fields}) VALUES({values}) RETURNING *"
        try:
            result = await self.conn.execute(
                query,
                node_info.hostname,
                node_info.event,
                node_info.seckey,
                node_info.pubkey,
                node_info.netkey,
            )
            if result is None:
                raise Exception("failed to insert leader0 node into from db")
            logger.debug(f"{node_info.hostname} info added: {result}")
        except Exception as e:
            raise Exception(f"leadership went wrong: {e}") from e

    async def fetch_leaders_host_info(self) -> List[PeerNode]:
        filter_by = "hostname != $1 AND hostname ~ '^leader[0-9]+$'"
        query = f"SELECT (hostname, pubkey) FROM voting_node WHERE {filter_by}"
        result = await self.conn.fetch(query, get_hostname())
        if result is None:
            raise Exception("db peer node error")
        logger.debug(f"peers node info retrieved from db: {len(result)}")
        rows = []
        for r in result:
            hostname, pubkey = r["row"]
            rows.append(PeerNode(hostname, pubkey))
        return rows

    async def fetch_proposals(self) -> List[Proposal]:
        sort_by = "id ASC"
        query = f"SELECT * FROM proposal ORDER BY {sort_by}"
        result = await self.conn.fetch(query)
        if result is None:
            raise Exception("proposals db error")
        logger.debug(f"proposals retrieved from db: {len(result)}")
        rows = []
        for r in result:
            rows.append(Proposal(**dict(r)))
        return rows

    async def insert_block0_info(
        self, event_row_id: int, block0_bytes: bytes, block0_hash: str
    ):
        # insert the hostname row into the voting_node table
        columns = "block0 = $1, block0_hash = $2"
        condition = "row_id = $3"
        returning = "name"
        query = f"UPDATE event SET {columns} WHERE {condition} RETURNING {returning}"
        try:
            result = await self.conn.execute(
                query, block0_bytes, block0_hash, event_row_id
            )
            if result is None:
                raise Exception("failed to insert block0 info from db")
            logger.debug(f"block0 info added to event: {result}")
        except Exception as e:
            raise Exception(f"inserting block0 info went wrong: {e}") from e
