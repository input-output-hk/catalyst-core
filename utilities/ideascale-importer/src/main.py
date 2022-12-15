import asyncio
import typer
from typing import Optional

from importer import Importer


app = typer.Typer(add_completion=False)


@app.command()
def import_all(
    api_token: str = typer.Option(..., help="IdeaScale API token"),
    database_url: str = typer.Option(..., help="Postgres database URL"),
    election_id: Optional[int] = typer.Option(
        None,
        help="Database row id of the election which data will be imported",
    ),
    campaign_group_id: Optional[int] = typer.Option(
        None,
        help="IdeaScale campaign group id for the election which data will be imported",
    ),
    stage_id: Optional[int] = typer.Option(
        None,
        help="IdeaScale stage id for from which proposal data will be imported",
    ),
):
    """
    Import all election data from IdeaScale for a given election
    """
    async def inner(
        election_id: Optional[int],
        campaign_group_id: Optional[int],
        stage_id: Optional[int],
    ):
        importer = Importer(
            api_token,
            database_url,
            election_id,
            campaign_group_id,
            stage_id
        )

        await importer.connect()
        await importer.import_all()
        await importer.close()

    asyncio.run(inner(election_id, campaign_group_id, stage_id))


if __name__ == "__main__":
    app()
