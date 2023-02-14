import click
import uvicorn
from . import api, node


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
    help="Host for the voting node API. If left unset it will look for the env var VOTING_HOST. If no host is found, the default value is: 0.0.0.0",
)
@click.option(
    "--port",
    envvar="VOTING_PORT",
    default=8000,
    help="Port for the voting node API. If left unset it will look for the env var VOTING_PORT. If no port is found, the default value is: 8000",
)
@click.option(
    "--log-level",
    envvar="VOTING_LOG_LEVEL",
    default="info",
    type=click.Choice(["info", "debug", "warn", "error", "trace"]),
    help="Sets the level for logs in the voting node. If left unset it will look for the env var VOTING_LOG_LEVEL. If no level is found, the default value is: info",
)
@click.option(
    "--database-url",
    envvar="DATABASE_URL",
    default="postgres://localhost/CatalystDev",
    help="Sets the URL for the database. Default: postgres://localhost/CatalystDev",
)
@click.option(
    "--storage",
    envvar="VOTING_NODE_STORAGE",
    default="./storage",
    help="Sets the location for the voting node's storage",
)
@click.option(
    "--jormungandr-path",
    envvar="JORMUNGANDR_PATH",
    default="jormungandr",
)
@click.option("--jcli-path", envvar="JCLI_PATH", default="~/.cargo/bin/jcli")
def start(host, port, log_level, database_url, storage, jormungandr_path, jcli_path):
    click.echo("Starting...")
    click.echo(f"host={host}")
    click.echo(f"port={port}")
    click.echo(f"log-level={log_level}")
    click.echo(f"database-url={database_url}")
    click.echo(f"storage={storage}")

    api_config = uvicorn.Config(api.app, host=host, port=port, log_level=log_level)
    jorm_config = node.JormConfig(jormungandr_path, jcli_path, storage)

    voting_node = node.VotingNode(api_config, jorm_config, database_url)
    voting_node.start()


# this groups commands in the main 'cli' group
cli.add_command(start)
