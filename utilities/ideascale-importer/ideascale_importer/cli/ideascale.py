"""IdeaScale CLI commands."""

import asyncio
from typing import Optional, List
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
    campaign_group_id: int = typer.Option(
        ...,
        envvar="IDEASCALE_CAMPAIGN_GROUP",
        help="IdeaScale campaign group id for the event which data will be imported",
    ),
    stage_ids: List[int] = typer.Option(
        ...,
        envvar="IDEASCALE_STAGE_ID",
        help="IdeaScale stage ids for from which proposal data will be imported",
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
        "text",
        envvar="IDEASCALE_LOG_FORMAT",
        help="Log format",
    ),
    ideascale_api_url: str = typer.Option(
        Client.DEFAULT_API_URL,
        envvar="IDEASCALE_API_URL",
        help="IdeaScale API URL",
    ),
):
    """Import all event data from IdeaScale for a given event."""
    configure_logger(log_level, log_format)

    async def inner(
        event_id: int,
        campaign_group_id: int,
        stage_ids: [int],
        proposals_scores_csv_path: Optional[str],
        ideascale_api_url: str,
    ):
        importer = Importer(
            api_token,
            database_url,
            None,
            event_id,
            campaign_group_id,
            stage_ids,
            proposals_scores_csv_path,
            ideascale_api_url,
        )

        try:
            await importer.connect()
            await importer.run()
            await importer.close()
        except Exception as e:
            logger.error(e)

    asyncio.run(inner(event_id, campaign_group_id, stage_ids, proposals_scores_csv, ideascale_api_url))
