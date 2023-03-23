import asyncio
from typing import Optional
import typer

from ideascale_importer.ideascale.importer import Importer

app = typer.Typer(add_completion=False)


@app.command()
def import_all(
    api_token: str = typer.Option(..., help="IdeaScale API token"),
    database_url: str = typer.Option(..., help="Postgres database URL"),
    event_id: Optional[int] = typer.Option(
        None,
        help="Database row id of the event which data will be imported",
    ),
    campaign_group_id: Optional[int] = typer.Option(
        None,
        help="IdeaScale campaign group id for the event which data will be imported",
    ),
    stage_id: Optional[int] = typer.Option(
        None,
        help="IdeaScale stage id for from which proposal data will be imported",
    ),
    proposals_scores_csv: Optional[str] = typer.Option(
        None,
        help="CSV file containing proposals impact scores",
    ),
):
    """
    Import all event data from IdeaScale for a given event
    """

    async def inner(
        event_id: Optional[int],
        campaign_group_id: Optional[int],
        stage_id: Optional[int],
        proposals_scores_csv_path: Optional[str]
    ):
        importer = Importer(
            api_token,
            database_url,
            None,
            event_id,
            campaign_group_id,
            stage_id,
            proposals_scores_csv_path,
        )

        await importer.connect()
        await importer.import_all()
        await importer.close()

    asyncio.run(inner(event_id, campaign_group_id, stage_id, proposals_scores_csv))
