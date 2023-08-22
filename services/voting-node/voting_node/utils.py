"""Utitilies for managing voting node data."""
import base64
import calendar
import re
import secrets
import socket
from pathlib import Path
from re import Match
from typing import Final, Literal

import yaml
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from loguru import logger
from pydantic.dataclasses import dataclass

from .jcli import JCli
from .models import Event, Genesis, LeaderHostInfo, NodeConfig
from .models.committee import Committee, CommitteeMember, CommunicationKeys, MemberKeys, WalletKeys
from .templates import (
    GENESIS_YAML,
    NODE_CONFIG_FOLLOWER,
    NODE_CONFIG_LEADER,
    NODE_CONFIG_LEADER0,
)

LEADER_REGEX: Final = r"^leader[0-9]+$"
"""Regex expression to determine a node is a leader"""
LEADERSHIP_REGEX: Final = r"^(leader|follower)([0-9]+)$"
"""Regex expression to determine a node's leadership and number"""
HASH_ITERATIONS: Final = 480000
"""Hash iterations performed in key derivation."""
SALT_BYTES: Final = 16
"""Size in bytes of encryption salt."""
KDF_LENGTH: Final = 32
"""Length in bytes uses in key derivation."""


def get_hostname() -> str:
    """Get the voting node hostname."""
    return socket.gethostname()


def get_hostname_addr(hostname: str | None = None) -> str:
    """Get the voting node hostname."""
    if hostname is None:
        hostname = get_hostname()
    return socket.gethostbyname(hostname)


async def get_network_secret(secret_file: Path, jcli_path: str) -> str:
    """Look for the secret_file and returns the secret.

    If the file doesn't exist, a new secret is generated and written to the file.
    """
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
            jcli_exec = JCli(jcli_path)
            secret = await jcli_exec.key_generate(secret_type="ed25519")
            # write the key to the file
            secret_file.open("w").write(secret)
            # save the key and the path to the file
            return secret
        except Exception as e:
            raise e


@dataclass
class NodeRole:
    """Represents the role a node is assuming."""

    name: Literal["leader", "follower"]
    n: int


def parse_node_role(
    s: str,
) -> NodeRole:
    """Parse a node role from a string."""
    res = re.match(LEADERSHIP_REGEX, s)
    exc = Exception(f"Role string '{s}' must conform to '{LEADERSHIP_REGEX}'")
    if res is None:
        raise exc
    match res.groups():
        case ("leader", n):
            return NodeRole("leader", int(n))
        case ("follower", n):
            return NodeRole("follower", int(n))
        case _:
            raise exc


def leader0_node_config(
    listen_rest: str,
    listen_jrpc: str,
    listen_p2p: str,
    trusted_peers,
    storage: Path,
    topology_key: Path,
) -> NodeConfig:
    """Configure a leader0 node from template."""
    node_config_dict = yaml.safe_load(NODE_CONFIG_LEADER0)
    node_config_dict["storage"] = f"{storage.absolute()}"
    node_config_dict["rest"]["listen"] = listen_rest
    node_config_dict["jrpc"]["listen"] = listen_jrpc
    node_config_dict["p2p"]["bootstrap"]["node_key_file"] = f"{topology_key.absolute()}"
    node_config_dict["p2p"]["bootstrap"]["trusted_peers"] = trusted_peers
    node_config_dict["p2p"]["connection"]["public_address"] = listen_p2p
    # these settings are special to leader0
    node_config_dict["bootstrap_from_trusted_peers"] = False
    node_config_dict["skip_bootstrap"] = True

    node_config = NodeConfig(node_config_dict)

    return node_config


def leader_node_config(
    listen_rest: str,
    listen_jrpc: str,
    listen_p2p: str,
    trusted_peers: list[dict],
    storage: Path,
    topology_key: Path,
) -> NodeConfig:
    """Configure a leader node from template."""
    node_config_dict = yaml.safe_load(NODE_CONFIG_LEADER)

    node_config_dict["storage"] = f"{storage.absolute()}"
    node_config_dict["rest"]["listen"] = listen_rest
    node_config_dict["jrpc"]["listen"] = listen_jrpc
    node_config_dict["p2p"]["bootstrap"]["node_key_file"] = f"{topology_key.absolute()}"
    node_config_dict["p2p"]["bootstrap"]["trusted_peers"] = trusted_peers
    node_config_dict["p2p"]["connection"]["public_address"] = listen_p2p
    # follower and leader nodes use these settings
    node_config_dict["bootstrap_from_trusted_peers"] = True
    node_config_dict["skip_bootstrap"] = False

    node_config = NodeConfig(node_config_dict)

    return node_config


def follower_node_config(
    listen_rest: str,
    listen_jrpc: str,
    listen_p2p: str,
    trusted_peers: list[dict],
    storage: Path,
    topology_key: Path,
) -> NodeConfig:
    """Configure a follower node from template."""
    node_config_dict = yaml.safe_load(NODE_CONFIG_FOLLOWER)

    node_config_dict["storage"] = f"{storage.absolute()}"
    node_config_dict["rest"]["listen"] = listen_rest
    node_config_dict["jrpc"]["listen"] = listen_jrpc
    node_config_dict["p2p"]["bootstrap"]["node_key_file"] = f"{topology_key.absolute()}"
    node_config_dict["p2p"]["bootstrap"]["trusted_peers"] = trusted_peers
    node_config_dict["p2p"]["connection"]["public_address"] = listen_p2p
    # follower nodes are configured to keep a persistent log
    # get the path to the log directory, create it if necessary
    persistent_log = storage.joinpath("persistent_log")
    persistent_log.mkdir(parents=True, exist_ok=True)
    node_config_dict["mempool"]["persistent_log"]["dir"] = f"{persistent_log.absolute()}"
    # follower and leader nodes use these settings
    node_config_dict["bootstrap_from_trusted_peers"] = True
    node_config_dict["skip_bootstrap"] = False

    node_config = NodeConfig(node_config_dict)

    return node_config


def make_node_config(
    leadership: NodeRole,
    listen_rest: str,
    listen_jrpc: str,
    listen_p2p: str,
    trusted_peers: list[dict],
    storage: Path,
    topology_key: Path,
) -> NodeConfig:
    """Configure a node from template, depending on its leadership and number."""
    match leadership:
        case NodeRole("leader", 0):
            return leader0_node_config(
                listen_rest,
                listen_jrpc,
                listen_p2p,
                trusted_peers,
                storage,
                topology_key,
            )
        case NodeRole("leader", _):
            return leader_node_config(
                listen_rest,
                listen_jrpc,
                listen_p2p,
                trusted_peers,
                storage,
                topology_key,
            )
        case NodeRole("follower", _):
            return follower_node_config(
                listen_rest,
                listen_jrpc,
                listen_p2p,
                trusted_peers,
                storage,
                topology_key,
            )
        case _:
            raise Exception("something odd happened creating node_config.yaml")


async def create_wallet_keyset(jcli: JCli) -> WalletKeys:
    """Generate Ed25519 keys used for wallet addresses."""
    wsk = await jcli.key_generate()
    wpk = await jcli.key_to_public(wsk)
    wid = await jcli.key_to_bytes(wpk)
    return WalletKeys(seckey=wsk, pubkey=wpk, hex_encoded=wid)


async def create_communication_keys(jcli: JCli) -> CommunicationKeys:
    """Create the communication keys for a committee member.

    The public communication keys are used to create the member keys.
    """
    comm_sk = await jcli.votes_committee_communication_key_generate()
    comm_pk = await jcli.votes_committee_communication_key_to_public(comm_sk)
    return CommunicationKeys(seckey=comm_sk, pubkey=comm_pk)


async def create_committee_member_keys(
    jcli: JCli,
    threshold: int,
    crs: str,
    index: int,
    keys: list[str],
) -> MemberKeys:
    """Return a tuple with the committee member key pair.

    `threshold` is the least number of members needed to perform a tally.
    `index` is the zero-based index of the member, ranging from `0 <= index < committee_size`.
    `crs` is the Common Reference String shared by all member keys.
    `keys` are the public communication keys of every committee member.
    """
    logger.info(f"creating committee member_{index} keys, threshold is {threshold}")
    member_pk = await jcli.votes_committee_member_key_generate(keys, crs, index, threshold)
    logger.debug(f"member_{index}_sk created")
    member_sk = await jcli.votes_committee_member_key_to_public(member_pk)
    logger.debug(f"member_{index}_pk created: {member_pk}")
    return MemberKeys(seckey=member_sk, pubkey=member_sk)


async def create_committee(jcli: JCli, event_id: int, size: int, threshold: int, crs: str) -> Committee:
    """Return a Committee.

    `committee_id` is the hex-encoded public key of the Committee wallet.
    `size` is equal to the number of committee members. If set to 0, voting and tallying will be public.
    `threshold` is the minimum number of committee members needed to carry out the tally.
    `crs` is the common reference string shared by all member keys.
    """
    logger.debug("creating committee wallet info")
    committee_wallet = await create_wallet_keyset(jcli)
    committee_pk = committee_wallet.seckey
    committee_id = committee_wallet.hex_encoded
    communication_keys = [await create_communication_keys(jcli) for _ in range(size)]

    def comm_pk(kp: CommunicationKeys) -> str:
        match kp:
            case CommunicationKeys(pubkey=pk):
                return pk

    comm_pks = [comm_pk(comm_keys) for comm_keys in communication_keys]
    member_keys = [await create_committee_member_keys(jcli, threshold, crs, idx, comm_pks) for idx in range(size)]

    def member_pk(kp: MemberKeys) -> str:
        match kp:
            case MemberKeys(pubkey=pk):
                return pk

    member_pks = [member_pk(mks) for mks in member_keys]
    members = [
        CommitteeMember(index=idx, communication_keys=communication_keys[idx], member_keys=member_keys[idx]) for idx in range(size)
    ]
    election_key = await jcli.votes_election_key(member_pks)
    return Committee(
        event_id=event_id,
        size=size,
        threshold=threshold,
        crs=crs,
        committee_pk=committee_pk,
        committee_id=committee_id,
        members=members,
        election_key=election_key,
    )


def make_genesis_content(event: Event, peers: list[LeaderHostInfo], committee_ids: list[str]) -> Genesis:
    """Generate a genesis file."""
    voting_start = event.get_voting_start()
    genesis = yaml.safe_load(GENESIS_YAML)
    consensus_leader_ids = [peer.consensus_leader_id for peer in peers]
    # modify the template with the proper settings
    genesis["blockchain_configuration"]["block0_date"] = int(calendar.timegm(voting_start.utctimetuple()))
    genesis["blockchain_configuration"]["consensus_leader_ids"] = consensus_leader_ids
    genesis["blockchain_configuration"]["committees"] = committee_ids

    return Genesis(genesis)


async def make_block0(jcli_path: str, storage: Path, genesis_path: Path) -> tuple[Path, str]:
    """Make the binary content of block0 from the given genesis.yaml path."""
    block0_path = storage.joinpath("block0.bin")
    jcli_exec = JCli(jcli_path)
    await jcli_exec.genesis_encode(block0_path, genesis_path)
    hash = await make_block0_hash(jcli_path, block0_path)
    return (block0_path, hash)


async def make_block0_hash(jcli_path: str, block0_path: Path) -> str:
    """Return the hash of the block0 binary from the given block0.bin path."""
    jcli_exec = JCli(jcli_path)
    return await jcli_exec.genesis_hash(block0_path)


def encrypt_secret(secret: str, password: str) -> str:
    """Encrypt secret with with Fernet."""
    salt = secrets.token_bytes(SALT_BYTES)
    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA256(),
        length=KDF_LENGTH,
        salt=salt,
        iterations=HASH_ITERATIONS,
    )
    key = base64.urlsafe_b64encode(kdf.derive(password.encode()))
    f = Fernet(key)
    encrypted = f.encrypt(secret.encode())
    b64 = base64.urlsafe_b64encode(salt + encrypted)
    return b64.decode()


def decrypt_secret(encrypted: str, password: str) -> str:
    """Decrypt secret with with Fernet."""
    b64 = base64.urlsafe_b64decode(encrypted)
    salt = b64[:SALT_BYTES]
    encrypted_secret = b64[SALT_BYTES:]
    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA256(),
        length=KDF_LENGTH,
        salt=salt,
        iterations=HASH_ITERATIONS,
    )
    key = base64.urlsafe_b64encode(kdf.derive(password.encode()))
    f = Fernet(key)
    secret = f.decrypt(encrypted_secret)
    return secret.decode()
