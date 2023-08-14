import asyncio
import typer

from ideascale_importer.reviews_importer import Client
from ideascale_importer.reviews_importer import Importer
from ideascale_importer.utils import configure_logger
from loguru import logger

app = typer.Typer(add_completion=False)

@app.command(name="import")
def import_reviews(
    ideascale_url: str = typer.Option(
        Client.DEFAULT_API_URL,
        envvar="IDEASCALE_API_URL",
        help="IdeaScale API URL",
    ),
    email: str = typer.Option(
        ...,
        help="Ideascale user's email address (needs admin access)",
    ),
    password: str = typer.Option(
        ...,
        help="Ideascale user's password (needs admin access)",
    ),
    funnel_id: int = typer.Option(
        ...,
        help="Ideascale campaign funnel's id",
    ),
    out_dir: str = typer.Option(
        ...,
        help="Output directoy for storing review's files",
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
            email,
            password,
            funnel_id,
            out_dir,
        )

        try:
            await importer.connect()
            await importer.run()
            await importer.close()
        except Exception as e:
            logger.error(e)

    asyncio.run(inner())