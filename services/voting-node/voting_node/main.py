from typing import Final

import click
import uvicorn

from . import api, logs, service
from .models import ServiceSettings

# Environment variables
VOTING_HOST: Final = "VOTING_HOST"
VOTING_PORT: Final = "VOTING_PORT"
VOTING_LOG_LEVEL: Final = "VOTING_LOG_LEVEL"
VOTING_NODE_STORAGE: Final = "VOTING_NODE_STORAGE"
IS_NODE_RELOADABLE: Final = "IS_NODE_RELOADABLE"
EVENTDB_URL: Final = "EVENTDB_URL"
JORM_PATH: Final = "JORM_PATH"
JORM_PORT_REST: Final = "JORM_PORT_REST"
JORM_PORT_JRPC: Final = "JORM_PORT_JRPC"
JORM_PORT_P2P: Final = "JORM_PORT_P2P"


@click.group()
def cli():
    """Main CLI entry point."""


@click.command()
@click.option(
    "--reloadable",
    is_flag=True,
    envvar="IS_NODE_RELOADABLE",
    help="""\
    Enables the node to reload when it detects changes to the current Voting Event.\
    If not set, the node will still detect changes to the Voting Event, but will use\
    the configuration it has, emitting warnings in the logs.
    """,
)
@click.option(
    "--host",
    envvar=VOTING_HOST,
    default="0.0.0.0",
    help="""\
    Host for the voting node API. If left unset it will look for VOTING_HOST.\
    If no host is found, the default value is: 0.0.0.0""",
)
@click.option(
    "--port",
    envvar=VOTING_PORT,
    default=8000,
    help="""\
    Port for the voting node API. If left unset it will look for VOTING_PORT.\
    If no port is found, the default value is: 8000""",
)
@click.option(
    "--log-level",
    envvar=VOTING_LOG_LEVEL,
    default="info",
    type=click.Choice(["info", "debug", "warn", "error", "trace"]),
    help="""\
    Sets the level for logs in the voting node. If left unset it will look for\
    VOTING_LOG_LEVEL. If no level is found, the default value is: info""",
)
@click.option(
    "--database-url",
    envvar=EVENTDB_URL,
    default="postgres://localhost/CatalystEventDev",
    help="""\
    Sets the URL for the database. Default: postgres://localhost/CatalystEventDev""",
)
@click.option(
    "--node-storage",
    envvar=VOTING_NODE_STORAGE,
    default="./node_storage",
    help="Sets the path to the voting node storage directory",
)
@click.option(
    "--jorm-path",
    envvar=JORM_PATH,
    default="jormungandr",
    help="""\
    Path to the 'jormungandr' executable.
    """,
)
@click.option(
    "--jorm-port-rest",
    envvar=JORM_PORT_REST,
    default=10080,
    help="""\
    jormungandr REST listening port
    """,
)
@click.option(
    "--jorm-port-jrpc",
    envvar=JORM_PORT_JRPC,
    default=10085,
    help="""\
    jormungandr JRPC listening port
    """,
)
@click.option(
    "--jorm-port-p2p",
    envvar=JORM_PORT_P2P,
    default=10090,
    help="""\
    jormungandr P2P listening port
    """,
)
@click.option(
    "--jcli-path",
    envvar="JCLI_PATH",
    default="jcli",
    help="""\
    Path to the 'jcli' executable.
    """,
)
def start(
    reloadable,
    host,
    port,
    log_level,
    database_url,
    node_storage,
    jorm_path,
    jcli_path,
    jorm_port_rest,
    jorm_port_jrpc,
    jorm_port_p2p,
):
    """Starts the voting node."""
    logs.configLogger(log_level)
    click.echo(f"reloadable={reloadable}")

    api_config = uvicorn.Config(api.app, host=host, port=port, log_level=log_level)
    settings = ServiceSettings(
        jorm_port_rest,
        jorm_port_jrpc,
        jorm_port_p2p,
        node_storage,
        jcli_path,
        jorm_path,
        database_url,
        reloadable,
    )

    voting = service.VotingService(api_config, settings)
    voting.start()


# this groups commands in the main 'cli' group
cli.add_command(start)

if __name__ == "__main__":
    cli()
