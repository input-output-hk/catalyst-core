import asyncio
import typer
import traceback
from typing import List
from loguru import logger

from ideascale_importer.reviews_manager.manager import ReviewsManager
from ideascale_importer.utils import configure_logger

app = typer.Typer(add_completion=False)

@app.command(name="import-reviews")
def import_reviews(
    ideascale_url: str = typer.Option(
        ...,
        envvar="IDEASCALE_API_URL",
        help="IdeaScale API URL",
    ),
    database_url: str = typer.Option(
        ...,
        envvar="EVENTDB_URL",
        help="Postgres database URL"
    ),
    email: str = typer.Option(
        ...,
        envvar="IDEASCALE_EMAIL",
        help="Ideascale user's email address (needs admin access)",
    ),
    password: str = typer.Option(
        ...,
        envvar="IDEASCALE_PASSWORD",
        help="Ideascale user's password (needs admin access)",
    ),
    event_id: int = typer.Option(
        ...,
        help="Database row id of the event which data will be imported",
    ),
    api_token: str = typer.Option(
        ...,
        envvar="IDEASCALE_API_TOKEN",
        help="IdeaScale API token"
    ),
    allocations_path: str = typer.Option(
        ..., 
        help="allocations file"
    ),
    output_path: str = typer.Option(
        ...,
        help="output path"
    ),
    log_level: str = typer.Option(
        "info",
        envvar="REVIEWS_LOG_LEVEL",
        help="Log level",
    ),
    log_format: str = typer.Option(
        "json",
        envvar="REVIEWS_LOG_FORMAT",
        help="Log format",
    )
):
    """Import all reviews data from IdeaScale for the provided event."""
    configure_logger(log_level, log_format)

    async def inner():
        importer = ReviewsManager(
            ideascale_url=ideascale_url,
            database_url=database_url,
            email=email,
            password=password,
            api_token=api_token,
            event_id=event_id,
        )

        try:
            await importer.connect()
            await importer.import_reviews_run(
                allocations_path=allocations_path,
                output_path=output_path
                )
            await importer.close()
        except Exception as e:
            traceback.print_exc()
            logger.error(e)

    asyncio.run(inner())

@app.command(name="prepare-allocations")
def prepare_allocations(
    ideascale_url: str = typer.Option(
        ...,
        envvar="IDEASCALE_API_URL",
        help="IdeaScale API URL",
    ),
    database_url: str = typer.Option(
        ...,
        envvar="EVENTDB_URL",
        help="Postgres database URL"
    ),
    email: str = typer.Option(
        ...,
        envvar="IDEASCALE_EMAIL",
        help="Ideascale user's email address (needs admin access)",
    ),
    password: str = typer.Option(
        ...,
        envvar="IDEASCALE_PASSWORD",
        help="Ideascale user's password (needs admin access)",
    ),
    event_id: int = typer.Option(
        ...,
        help="Database row id of the event which data will be imported",
    ),
    api_token: str = typer.Option(
        ...,
        envvar="IDEASCALE_API_TOKEN",
        help="IdeaScale API token"
    ),
    pas_path: str = typer.Option(
        ..., 
        help="PAs file"
    ),
    output_path: str = typer.Option(
        ...,
        help="output path"
    ),
    log_level: str = typer.Option(
        "info",
        envvar="REVIEWS_LOG_LEVEL",
        help="Log level",
    ),
    log_format: str = typer.Option(
        "json",
        envvar="REVIEWS_LOG_FORMAT",
        help="Log format",
    )
):
    """Prepare allocations for the provided event."""
    configure_logger(log_level, log_format)

    async def inner():
        importer = ReviewsManager(
            ideascale_url=ideascale_url,
            database_url=database_url,
            email=email,
            password=password,
            api_token=api_token,
            event_id=event_id,
        )

        try:
            await importer.connect()
            await importer.generate_allocations_run(
                pas_path=pas_path,
                output_path=output_path
                )
            await importer.close()
        except Exception as e:
            traceback.print_exc()
            logger.error(e)

    asyncio.run(inner())