import re
import socket

from pathlib import Path
from typing import Final, Literal

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


def get_leadership_role_by_hostname(
    host_name: str,
) -> Literal["leader0", "leader", "follower"]:
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
