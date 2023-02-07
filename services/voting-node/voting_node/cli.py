import click
from . import app

@click.group()
@click.option('--debug/--no-debug', default=False)
@click.option('--hot-reload/--no-hot-reload', default=False)
def cli(debug, hot_reload):
    click.echo(f"debug-mode={debug}")
    click.echo(f"hot-reload-mode={hot_reload}")

@click.command()
@click.option('--host', envvar='VOTING_HOST', default='0.0.0.0')
@click.option('--port', envvar='VOTING_PORT', default=8000)
@click.option('--log-level', envvar='VOTING_LOG_LEVEL', default='info')
@click.option('--jormungandr-path', envvar='JORMUNGANDR_PATH', default='/home/saiba/.cargo/bin/jormungandr')
@click.option('--jcli-path', envvar='JCLI_PATH', default='/home/saiba/.cargo/bin/jcli')
def start(host, port, log_level, jormungandr_path, jcli_path):
    click.echo('Starting...')
    click.echo(f"host={host}")
    click.echo(f"port={port}")
    click.echo(f"log-level={log_level}")
    app.run(jormungandr_path=jormungandr_path, jcli_path=jcli_path, host=host, port=port, log_level=log_level)

cli.add_command(start)
