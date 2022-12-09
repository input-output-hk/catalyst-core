import asyncio
import rich.console
import rich.table
import typer

import ideascale

app = typer.Typer()

@app.command()
def list_campaigns(
    api_token: str = typer.Option(..., help="IdeaScale API token"),
    campaign_group_id: int = typer.Option(..., help="IdeaScale campaign group id"),
):
    """
    List campaigns from IdeaScale belonging to the given campaign group
    """
    async def inner():
        client = ideascale.client_with_progress(api_token)

        campaigns = []
        with client.request_progress_observer:
            campaigns = await client.campaigns(campaign_group_id)
        typer.echo()

        table = rich.table.Table("Id", "Name", title="Campaigns")

        campaigns.sort(key=lambda c: c.id, reverse=True)
        for c in campaigns:
            table.add_row(str(c.id), c.name)

        rich.console.Console().print(table)

    asyncio.run(inner())

@app.command()
def list_campaign_groups(api_token: str = typer.Option(..., help="IdeaScale API token")):
    """
    List campaign groups from IdeaScale
    """
    async def inner():
        client = ideascale.client_with_progress(api_token)

        groups = []
        with client.request_progress_observer:
            groups = await client.campaign_groups()
        typer.echo()

        table = rich.table.Table("Id", "Name", title="Campaign Groups")

        groups.sort(key=lambda g: g.id, reverse=True)
        for g in groups:
            table.add_row(str(g.id), g.name)

        rich.console.Console().print(table)

    asyncio.run(inner())

@app.command()
def list_campaign_ideas(
    api_token: str = typer.Option(..., help="IdeaScale API token"),
    campaign_id: int = typer.Option(..., help="IdeaScale campaign id"),
):
    """
    List ideas from IdeaScale beloging to a given campaign
    """
    async def inner():
        client = ideascale.client_with_progress(api_token)

        ideas = []
        with client.request_progress_observer:
            ideas = await client.campaign_ideas(campaign_id)
        typer.echo()

        table = rich.table.Table("Id", "Title", title="Ideas")

        ideas.sort(key=lambda i: i.id)
        for i in ideas:
            table.add_row(str(i.id), i.title)

        rich.console.Console().print(table)

    asyncio.run(inner())

@app.command()
def list_campaign_group_ideas(
    api_token: str = typer.Option(..., help="IdeaScale API token"),
    campaign_group_id: int = typer.Option(..., help="IdeaScale campaign group id"),
):
    """
    List ideas from IdeaScale beloging to a given campaign group
    """
    async def inner():
        client = ideascale.client_with_progress(api_token)

        with client.request_progress_observer:
            ideas = await client.campaign_group_ideas(campaign_group_id)

        typer.echo()

        table = rich.table.Table("Id", "Title", title="Ideas")

        ideas.sort(key=lambda i: i.id)
        for i in ideas:
            table.add_row(str(i.id), i.title)

        rich.console.Console().print(table)

    asyncio.run(inner())
