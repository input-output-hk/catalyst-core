import asyncio
import typer

from typing import List

from ideascale_importer.reviews_importer import FrontendClient
from ideascale_importer.reviews_importer import Importer
from ideascale_importer.utils import configure_logger
from loguru import logger

app = typer.Typer(add_completion=False)

@app.command(name="import")
def import_reviews(
    ideascale_url: str = typer.Option(
        FrontendClient.DEFAULT_API_URL,
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
        help="Ideascale user's email address (needs admin access)",
    ),
    password: str = typer.Option(
        ...,
        help="Ideascale user's password (needs admin access)",
    ),
    api_token: str = typer.Option(...,
        envvar="IDEASCALE_API_TOKEN",
        help="IdeaScale API token"
    ),
    funnel_id: int = typer.Option(
        ...,
        help="Ideascale campaign funnel's id",
    ),
    nr_allocations: List[int] = typer.Option(
        [30, 80], 
        help="Nr of proposal to allocate"
    ),
    pa_path: str = typer.Option(
        ...,
        help="PAs file"
    ),
    log_level: str = typer.Option(
        "info",
        envvar="REVIEWS_LOG_LEVEL",
        help="Log level",
    ),
    log_format: str = typer.Option(
        "text",
        envvar="REVIEWS_LOG_FORMAT",
        help="Log format",
    )
):
    """Import all reviews data from IdeaScale for a given funnel."""
    configure_logger(log_level, log_format)

    async def inner():
        importer = Importer(
            ideascale_url,
            database_url,
            email,
            password,
            api_token,
            funnel_id,
            nr_allocations,
            pa_path
        )

        try:
            await importer.connect()
            await importer.run()
            await importer.close()
        except Exception as e:
            logger.error(e)

    asyncio.run(inner())