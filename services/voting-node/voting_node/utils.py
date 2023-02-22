import datetime
import json
import re
import socket

from pathlib import Path
from typing import Dict, Final, Literal

from . import jcli, logs

# gets voting node logger
logger = logs.getLogger()


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


async def fetch_leader0_node_info(conn) -> Dict:
    filter_by = "hostname = $1"
    query = f"SELECT * FROM voting_nodes WHERE {filter_by}"
    result = await conn.fetchrow(query, get_hostname())
    if result is None:
        raise Exception("failed to fetch leader0 node info from db")
    return dict(result)


async def insert_leader0_node_info(conn, jcli_path: str):
    jcli_exec = jcli.JCli(jcli_path)
    hostname = get_hostname()
    logger.debug(f"generating {hostname} node info with jcli: {jcli_path}")
    seckey = await jcli_exec.seckey("ed25519")
    logger.debug("seckey generated")
    pubkey = await jcli_exec.pubkey(seckey)
    logger.debug("pubkey generated")
    netkey = await jcli_exec.seckey("ed25519")
    logger.debug("netkey generated")
    # this secret is not used, apparently, just to be safe
    consensus_secret = await jcli_exec.seckey("ed25519-extended")
    # the consensus_id is the public key
    consensus_id = await jcli_exec.pubkey(consensus_secret)
    logger.debug("consensus_id generated")
    # just to be safe, we store the pub and sec parts of the consensus_id
    extra = json.dumps(
        {"consensus_id": consensus_id, "consensus_secret": consensus_secret}
    )
    fields = "hostname, pubkey, seckey, netkey, extra"
    values = "$1, $2, $3, $4, $5"
    query = f"INSERT INTO voting_nodes({fields}) VALUES({values}) RETURNING *"
    try:
        result = await conn.execute(query, hostname, seckey, pubkey, netkey, extra)
        if result is None:
            raise Exception("failed to insert leader0 node into from db")
        logger.debug(f"{hostname} info added: {result}")
    except Exception as e:
        logger.error(f"leadership went wrong: {e}")
        raise e


def get_hostname() -> str:
    return socket.gethostname()


def get_leadership_role_by_hostname() -> Literal["leader0", "leader", "follower"]:
    host_name: str = get_hostname()
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
