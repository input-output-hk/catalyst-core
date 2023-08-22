"""Command-line Interface for the Voting Node Service.

Main entrypoint for executing the voting node service from the shell command-line.
"""

import click
import uvicorn
from ideascale_importer.utils import configure_logger
from . import api, service
from .envvar import (
    EVENTDB_URL,
    IS_NODE_RELOADABLE,
    JCLI_PATH,
    JORM_PATH,
    JORM_PORT_JRPC,
    JORM_PORT_P2P,
    JORM_PORT_REST,
    VOTING_HOST,
    VOTING_LOG_LEVEL,
    VOTING_LOG_FORMAT,
    VOTING_NODE_ROLE,
    VOTING_NODE_STORAGE,
    VOTING_PORT,
)
from .models import ServiceSettings


@click.group()
def voting_node_cli():
    """Deploy a jormungandr node for voting events."""


@click.command()
@click.option(
    "--reloadable",
    is_flag=True,
    envvar=IS_NODE_RELOADABLE,
    default=False,
    help=r"""Flag to enable the voting node to run in reloadable mode.

    When set, the node will reload its settings whenever changes to the current voting event are detected.
    Otherwise, the node will still detect changes in the event, but will not override its settings. Log warnings will be emitted.

    If left unset it will look for envvar `IS_NODE_RELOADABLE`.
    """,
)
@click.option(
    "--api-host",
    envvar=VOTING_HOST,
    default="0.0.0.0",
    help="""Host for the voting node API.

    If left unset it will look for envvar `VOTING_HOST`. If no host is found, the default value is: 0.0.0.0""",
)
@click.option(
    "--api-port",
    envvar=VOTING_PORT,
    default=8000,
    help="""Port for the voting node API.

    If left unset it will look for envvar `VOTING_PORT`. If no port is found, the default value is: 8000""",
)
@click.option(
    "--log-level",
    envvar=VOTING_LOG_LEVEL,
    default="info",
    type=click.Choice(["info", "debug", "warn", "error", "trace"]),
    help="""Set the level for logs in the voting node.

    If left unset it will look for envvar `VOTING_LOG_LEVEL`. If no level is found, the default value is: info""",
)
@click.option(
    "--log-format",
    envvar=VOTING_LOG_FORMAT,
    default="text",
    type=click.Choice(["text", "json"]),
    help="""Set the format for logs in the voting node.

    If left unset it will look for envvar `VOTING_LOG_FORMAT`. If no format is found, the default value is: text""",
)
@click.option(
    "--database-url",
    envvar=EVENTDB_URL,
    help="""Sets the URL for the database.

    Example: postgres://localhost/CatalystEventDev

    If left unset, it will look for envvar `EVENTDB_URL`.""",
)
@click.option(
    "--node-storage",
    envvar=VOTING_NODE_STORAGE,
    default="./node_storage",
    help="""Sets the path to the voting node storage directory.

    If left unset, it will look for envvar `JORM_PATH`.""",
)
@click.option(
    "--node-role",
    envvar=VOTING_NODE_ROLE,
    help="""Role which the node will assume (e.g. leader0).

    if let unset, it will look for envvar `VOTING_NODE_ROLE`.""",
)
@click.option(
    "--jorm-path",
    envvar=JORM_PATH,
    default="jormungandr",
    help="""Path to the 'jormungandr' executable.

    If left unset, it will look for envvar `JORM_PATH`.""",
)
@click.option(
    "--jorm-port-rest",
    envvar=JORM_PORT_REST,
    default=10080,
    help="""jormungandr REST listening port.

    If left unset, it will look for envvar `JORM_PORT_REST`.""",
)
@click.option(
    "--jorm-port-jrpc",
    envvar=JORM_PORT_JRPC,
    default=10085,
    help="""jormungandr JRPC listening port.

    If left unset, it will look for envvar `JORM_PORT_JRPC`.""",
)
@click.option(
    "--jorm-port-p2p",
    envvar=JORM_PORT_P2P,
    default=10090,
    help="""jormungandr P2P listening port.

    If left unset, it will look for envvar `JORM_PORT_P2P`.""",
)
@click.option(
    "--jcli-path",
    envvar=JCLI_PATH,
    default="jcli",
    help="""Path to the 'jcli' executable.

    If left unset, it will look for envvar `JCLI_PATH`.""",
)
def start(
    reloadable,
    api_host,
    api_port,
    log_level,
    log_format,
    database_url,
    node_storage,
    node_role,
    jorm_path,
    jcli_path,
    jorm_port_rest,
    jorm_port_jrpc,
    jorm_port_p2p,
):
    """Start the Voting Service."""
    configure_logger(log_level, log_format)

    api_config = uvicorn.Config(api.app, host=api_host, port=api_port, log_level=log_level)
    settings = ServiceSettings(
        jorm_port_rest,
        jorm_port_jrpc,
        jorm_port_p2p,
        node_storage,
        jcli_path,
        jorm_path,
        database_url,
        reloadable,
        node_role,
    )

    voting = service.VotingService(api_config, settings)
    voting.start()


# this groups commands in the main 'voting_node_cli' group
voting_node_cli.add_command(start)

if __name__ == "__main__":
    voting_node_cli()
