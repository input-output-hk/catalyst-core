import datetime
import re
import socket

from pathlib import Path
from typing import Dict, Final, Literal

from . import jcli, logs

# gets voting node logger
logger = logs.getLogger()


def get_hostname() -> str:
    """Gets the voting node hostname."""
    return socket.gethostname()


def get_hostname_addr(hostname: str | None = None) -> str:
    """Gets the voting node hostname."""
    if hostname is None:
        hostname = get_hostname()
    return socket.gethostbyname(hostname)


async def get_network_secret(secret_file: Path, jcli_path: str) -> str:
    """Looks for the secret_file and returns the secret. If ther file
    doesn't exist, a new secret is generated and written to the file."""
    # check for the file
    if secret_file.exists():
        # open and read it
        secret = secret_file.open("r").readline()
        logger.debug(f"found key: {secret} stored in {secret_file.absolute()}")
        # return the key and the path to the file
        return secret
    else:
        try:
            # run jcli to generate the secret key
            jcli_exec = jcli.JCli(jcli_path)
            secret = await jcli_exec.seckey(secret_type="ed25519")
            # write the key to the file
            secret_file.open("w").write(secret)
            # save the key and the path to the file
            return secret
        except Exception as e:
            raise e


async def fetch_election(conn) -> Dict:
    filter_by = "voting_start > $1"
    sort_by = "voting_start ASC"
    query = f"SELECT * FROM election WHERE {filter_by} ORDER BY {sort_by} LIMIT 1"
    now = datetime.datetime.utcnow()
    result = await conn.fetchrow(query, now)
    if result is None:
        raise Exception("failed to fetch election from db")
    return dict(result)


async def fetch_leader_node_info(conn) -> Dict:
    filter_by = "hostname = $1"
    query = f"SELECT * FROM voting_nodes WHERE {filter_by}"
    result = await conn.fetchrow(query, get_hostname())
    if result is None:
        raise Exception("failed to fetch leader node info from db")
    return dict(result)


async def fetch_peers_node_info(conn) -> Dict:
    filter_by = "hostname != $1 AND hostname ~ '^leader[0-9]+$'"
    query = f"SELECT (hostname, pubkey) FROM voting_nodes WHERE {filter_by}"
    result = await conn.fetch(query, get_hostname())
    if result is None:
        raise Exception("db peer node error")
    logger.debug(f"peers node info retrieved from db: {len(result)}")
    rows = []
    for r in result:
        hostname, pubkey = r["row"]
        ip_addr = None
        try:
            ip_addr = get_hostname_addr(hostname)
        except Exception:
            logger.warn(f"failed to get ip address for {hostname}")
        finally:
            rows.append((hostname, {"consensus_id": pubkey, "ip_addr": ip_addr}))
    return dict(rows)


async def insert_leader_node_info(conn, jcli_path: str):
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
        result = await conn.execute(query, hostname, seckey, pubkey, netkey)
        if result is None:
            raise Exception("failed to insert leader0 node into from db")
        logger.debug(f"{hostname} info added: {result}")
    except Exception as e:
        logger.error(f"leadership went wrong: {e}")
        raise e


def get_leadership_role_by_hostname(host_name: str) -> Literal["leader0", "leader", "follower"]:
    leader_regex: str = r"^(leader|follower)([0-9]+)$"
    ERR_MSG: Final[str] = f"hostname {host_name} needs to conform to '{leader_regex}'"
    res = re.match(leader_regex, host_name)
    if res is None:
        raise Exception(ERR_MSG)
    match res.groups():
        case ("leader", "0"):
            return "leader0"
        case ("leader", _):
            return "leader"
        case ("follower", _):
            return "follower"
        case _:
            raise Exception(ERR_MSG)
