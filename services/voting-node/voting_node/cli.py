import click
import uvicorn

from . import api, logs, node



@click.group()
@click.option("--debug/--no-debug", default=False)
@click.option("--hot-reload/--no-hot-reload", default=False)
def cli(debug, hot_reload):
    click.echo(f"debug-mode={debug}")
    click.echo(f"hot-reload-mode={hot_reload}")


@click.command()
@click.option(
    "--host",
    envvar="VOTING_HOST",
    default="0.0.0.0",
    help="""\
    Host for the voting node API. If left unset it will look for VOTING_HOST.\
    If no host is found, the default value is: 0.0.0.0""",
)
@click.option(
    "--port",
    envvar="VOTING_PORT",
    default=8000,
    help="""\
    Port for the voting node API. If left unset it will look for VOTING_PORT.\
    If no port is found, the default value is: 8000""",
)
@click.option(
    "--log-level",
    envvar="VOTING_LOG_LEVEL",
    default="info",
    type=click.Choice(["info", "debug", "warn", "error", "trace"]),
    help="""\
    Sets the level for logs in the voting node. If left unset it will look for\
    VOTING_LOG_LEVEL. If no level is found, the default value is: info""",
)
@click.option(
    "--database-url",
    envvar="DATABASE_URL",
    default="postgres://localhost/CatalystDev",
    help="Sets the URL for the database. Default: postgres://localhost/CatalystDev",
)
@click.option(
    "--node-storage",
    envvar="VOTING_NODE_STORAGE",
    default="./node_storage",
    help="Sets the location for the voting node's storage",
)
@click.option(
    "--jorm-path",
    envvar="JORM_PATH",
    default="jormungandr",
)
@click.option(
    "--jorm-rest-port",
    envvar="JORM_REST_PORT",
    default=10080,
)
@click.option(
    "--jorm-jrpc-port",
    envvar="JORM_JRPC_PORT",
    default=10085,
)
@click.option(
    "--jorm-p2p-port",
    envvar="JORM_P2P_PORT",
    default=10090,
)
@click.option("--jcli-path", envvar="JCLI_PATH", default="~/.cargo/bin/jcli")
def start(
    host,
    port,
    database_url,
    node_storage,
    log_level,
    jorm_path,
    jcli_path,
    jorm_rest_port,
    jorm_jrpc_port,
    jorm_p2p_port,
):
    logs.configLogger(log_level)

    logger = logs.getLogger()
    logger.debug("Executing: voting-node start")
    logger.info(f"host={host}")
    logger.info(f"port={port}")
    logger.info(f"database-url={database_url}")
    logger.info(f"node-storage={node_storage}")
    logger.info(f"log-level={log_level}")
    logger.info(f"jorm-path={jorm_path}")
    logger.info(f"jcli-path={jcli_path}")
    logger.info(f"rest-port={jorm_rest_port}")
    logger.info(f"jrpc-port={jorm_jrpc_port}")
    logger.info(f"p2p-port={jorm_p2p_port}")

    api_config = uvicorn.Config(api.app, host=host, port=port, log_level=log_level)
    jorm_config = node.JormConfig(
        jorm_path,
        jcli_path,
        node_storage,
        jorm_rest_port,
        jorm_jrpc_port,
        jorm_p2p_port,
    )

    voting_node = node.VotingNode(api_config, jorm_config, database_url)
    voting_node.start()


# this groups commands in the main 'cli' group
cli.add_command(start)
