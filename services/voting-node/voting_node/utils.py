import re
import socket
import yaml

from pathlib import Path
from typing import Dict, Final, Literal, List, Match, Tuple

from . import jcli
from .logs import getLogger
from .models import NodeConfig
from .templates import (
    NODE_CONFIG_LEADER_TEMPLATE,
    NODE_CONFIG_LEADER0_TEMPLATE,
    NODE_CONFIG_FOLLOWER_TEMPLATE,
)

# gets voting node logger
logger = getLogger()


"""Regex expression to determine a node's leadership and number"""
LEADER_REGEX: Final = r"^(leader|follower)([0-9]+)$"


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


def match_hostname_leadership_pattern(host_name: str) -> Match[str] | None:
    return re.match(LEADER_REGEX, host_name)


def get_leadership_role_n_number_by_hostname(
    host_name: str,
) -> Tuple[Literal["leader", "follower"], int]:
    res = match_hostname_leadership_pattern(host_name)
    exc = Exception(f"hostname {host_name} needs to conform to '{LEADER_REGEX}'")
    if res is None:
        raise exc
    match res.groups():
        case ("leader", n):
            return ("leader", int(n))
        case ("follower", n):
            return ("follower", int(n))
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
    """Configures a leader0 node from template."""
    node_config_dict = yaml.safe_load(NODE_CONFIG_LEADER0_TEMPLATE)
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
    trusted_peers: List[Dict],
    storage: Path,
    topology_key: Path,
) -> NodeConfig:
    """Configures a leader node from template."""
    node_config_dict = yaml.safe_load(NODE_CONFIG_LEADER_TEMPLATE)

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
    trusted_peers: List[Dict],
    storage: Path,
    topology_key: Path,
) -> NodeConfig:
    """Configures a follower node from template."""
    node_config_dict = yaml.safe_load(NODE_CONFIG_FOLLOWER_TEMPLATE)

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
    node_config_dict["mempool"]["persistent_log"][
        "dir"
    ] = f"{persistent_log.absolute()}"
    # follower and leader nodes use these settings
    node_config_dict["bootstrap_from_trusted_peers"] = True
    node_config_dict["skip_bootstrap"] = False

    node_config = NodeConfig(node_config_dict)

    return node_config


"""Configures a node from template, depending on its leadership and number."""


def make_node_config(
    leadership: Tuple[Literal["leader", "follower"], int],
    listen_rest: str,
    listen_jrpc: str,
    listen_p2p: str,
    trusted_peers: List[Dict],
    storage: Path,
    topology_key: Path,
) -> NodeConfig:
    match leadership:
        case ("leader", 0):
            return leader0_node_config(
                listen_rest,
                listen_jrpc,
                listen_p2p,
                trusted_peers,
                storage,
                topology_key,
            )
        case ("leader", _):
            return leader_node_config(
                listen_rest,
                listen_jrpc,
                listen_p2p,
                trusted_peers,
                storage,
                topology_key,
            )
        case ("follower", _):
            return follower_node_config(
                listen_rest,
                listen_jrpc,
                listen_p2p,
                trusted_peers,
                storage,
                topology_key,
            )
