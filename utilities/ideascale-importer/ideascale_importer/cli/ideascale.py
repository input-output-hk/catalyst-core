"""IdeaScale CLI commands."""

import asyncio
from pathlib import Path
from typing import Optional
import typer

from ideascale_importer.ideascale.client import Client
from ideascale_importer.ideascale.importer import Importer
from ideascale_importer.utils import configure_logger
from loguru import logger

app = typer.Typer(add_completion=False)


@app.command()
def import_all(
    api_token: str = typer.Option(..., envvar="IDEASCALE_API_TOKEN", help="IdeaScale API token"),
    database_url: str = typer.Option(..., envvar="EVENTDB_URL", help="Postgres database URL"),
    event_id: int = typer.Option(
        ...,
        help="Database row id of the event which data will be imported",
    ),
    proposals_scores_csv: Optional[str] = typer.Option(
        None,
        help="CSV file containing proposals impact scores",
    ),
    log_level: str = typer.Option(
        "info",
        envvar="IDEASCALE_LOG_LEVEL",
        help="Log level",
    ),
    log_format: str = typer.Option(
        "json",
        envvar="IDEASCALE_LOG_FORMAT",
        help="Log format",
    ),
    ideascale_api_url: str = typer.Option(
        Client.DEFAULT_API_URL,
        envvar="IDEASCALE_API_URL",
        help="IdeaScale API URL",
    ),
    output_dir: Optional[str] = typer.Option(
        default=None, envvar="IDEASCALE_OUTPUT_DIR", help="Output directory for generated files"
    ),
):
    """Import all event data from IdeaScale for a given event."""
    configure_logger(log_level, log_format)

    async def inner(
        event_id: int,
        proposals_scores_csv_path: Optional[str],
        ideascale_api_url: str,
        output_dir: Optional[str]
    ):
        # check if output_dir path exists, or create otherwise
        if output_dir is None:
            logger.info("No output directory was defined.")
        else:
            output_dir = Path(output_dir)
            output_dir.mkdir(exist_ok=True, parents=True)
            logger.info(f"Output directory for artifacts: {output_dir}")

        importer = Importer(
            api_token,
            database_url,
            event_id,
            proposals_scores_csv_path,
            ideascale_api_url,
            output_dir
        )

        try:
            await importer.connect()
            await importer.run()
            await importer.close()
        except Exception as e:
            logger.error(e)

    asyncio.run(inner(event_id, proposals_scores_csv, ideascale_api_url, output_dir))
