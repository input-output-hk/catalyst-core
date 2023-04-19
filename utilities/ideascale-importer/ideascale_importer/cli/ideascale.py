"""IdeaScale CLI commands."""

import asyncio
from typing import Optional
import typer

from ideascale_importer.ideascale.client import Client
from ideascale_importer.ideascale.importer import Importer
from ideascale_importer.utils import configure_logger

app = typer.Typer(add_completion=False)


@app.command()
def import_all(
    api_token: str = typer.Option(..., help="IdeaScale API token"),
    database_url: str = typer.Option(..., help="Postgres database URL"),
    event_id: int = typer.Option(
        ...,
        help="Database row id of the event which data will be imported",
    ),
    campaign_group_id: int = typer.Option(
        ...,
        help="IdeaScale campaign group id for the event which data will be imported",
    ),
    stage_id: int = typer.Option(
        ...,
        help="IdeaScale stage id for from which proposal data will be imported",
    ),
    proposals_scores_csv: Optional[str] = typer.Option(
        None,
        help="CSV file containing proposals impact scores",
    ),
    log_level: str = typer.Option(
        "info",
        help="Log level",
    ),
    log_format: str = typer.Option(
        "text",
        help="Log format",
    ),
    ideascale_api_url: str = typer.Option(
        Client.DEFAULT_API_URL,
        help="IdeaScale API URL",
    ),
):
    """Import all event data from IdeaScale for a given event."""
    configure_logger(log_level, log_format)

    configure_logger(log_level, log_format)

    async def inner(
        event_id: int,
        campaign_group_id: int,
        stage_id: int,
        proposals_scores_csv_path: Optional[str],
        ideascale_api_url: str,
    ):
        importer = Importer(
            api_token,
            database_url,
            None,
            event_id,
            campaign_group_id,
            stage_id,
            proposals_scores_csv_path,
            ideascale_api_url,
        )

        await importer.connect()
        await importer.run()
        await importer.close()

    asyncio.run(inner(event_id, campaign_group_id, stage_id, proposals_scores_csv, ideascale_api_url))
