import asyncpg
import datetime
from typing import Any, List

from voting_node.nodes import NodeInfo, PeerNode

from . import logs, jcli
from .utils import get_hostname, get_hostname_addr

# gets voting node logger
logger = logs.getLogger()


class ElectionDb(object):
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
        pass

    async def fetch_upcoming_election(self):
        filter_by = "voting_start > $1"
        sort_by = "voting_start ASC"
        query = f"SELECT * FROM election WHERE {filter_by} ORDER BY {sort_by} LIMIT 1"
        now = datetime.datetime.utcnow()
        result = await self.conn.fetchrow(query, now)
        if result is None:
            raise Exception("failed to fetch election from db")
        return result

    async def fetch_leader_node_info(self) -> NodeInfo:
        filter_by = "hostname = $1"
        query = f"SELECT * FROM voting_nodes WHERE {filter_by}"
        result = await self.conn.fetchrow(query, get_hostname())
        if result is None:
            raise Exception("failed to fetch leader node info from db")
        node_info = NodeInfo(
            result["hostname"], result["seckey"], result["pubkey"], result["netkey"]
        )
        return node_info

    async def insert_leader_node_info(self, jcli_path: str):
        hostname = get_hostname()
        logger.debug(f"generating {hostname} node info with jcli: {jcli_path}")
        # use JCli to make calls
        jcli_exec = jcli.JCli(jcli_path)
        # generate the keys
        seckey = await jcli_exec.seckey("ed25519")
        logger.debug("seckey generated")
        pubkey = await jcli_exec.pubkey(seckey)
        logger.debug("pubkey generated")
        netkey = await jcli_exec.seckey("ed25519")
        logger.debug("netkey generated")

        # insert the hostname row into the voting_nodes table
        fields = "hostname, seckey, pubkey, netkey"
        values = "$1, $2, $3, $4"
        query = f"INSERT INTO voting_nodes({fields}) VALUES({values}) RETURNING *"
        try:
            result = await self.conn.execute(query, hostname, seckey, pubkey, netkey)
            if result is None:
                raise Exception("failed to insert leader0 node into from db")
            logger.debug(f"{hostname} info added: {result}")
        except Exception as e:
            logger.error(f"leadership went wrong: {e}")
            raise e

    async def fetch_peers_node_info(self) -> List[PeerNode]:
        filter_by = "hostname != $1 AND hostname ~ '^leader[0-9]+$'"
        query = f"SELECT (hostname, pubkey) FROM voting_nodes WHERE {filter_by}"
        result = await self.conn.fetch(query, get_hostname())
        if result is None:
            raise Exception("db peer node error")
        logger.debug(f"peers node info retrieved from db: {len(result)}")
        rows = []
        for r in result:
            hostname, pubkey = r["row"]
            try:
                ip_addr = get_hostname_addr(hostname)
                rows.append(PeerNode(hostname, ip_addr, pubkey))
            except:
                raise Exception(f"failed to get ip address for {hostname}")
        return rows
