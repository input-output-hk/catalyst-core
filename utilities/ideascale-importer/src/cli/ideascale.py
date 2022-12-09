import asyncio
from tabulate import tabulate
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
        with client.request_progress_observer.progress:
            campaigns = await client.campaigns(campaign_group_id)
        typer.echo()

        table = [[c.id, c.name] for c in campaigns]
        table.sort(key=lambda i: i[0], reverse=True)

        typer.echo(tabulate(table, headers=["Id", "Name"]))

    asyncio.run(inner())

@app.command()
def list_campaign_groups(api_token: str = typer.Option(..., help="IdeaScale API token")):
    """
    List campaign groups from IdeaScale
    """
    async def inner():
        client = ideascale.client_with_progress(api_token)

        groups = []
        with client.request_progress_observer.progress:
            groups = await client.campaign_groups()
        typer.echo()

        table = [[g.id, g.name] for g in groups]
        table.sort(key=lambda i: i[0], reverse=True)

        typer.echo(tabulate(table, headers=["Id", "Name"]))

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
        with client.request_progress_observer.progress:
            ideas = await client.campaign_ideas(campaign_id)
        typer.echo()

        table = [[i.id, i.title] for i in ideas]
        table.sort(key=lambda i: i[0])

        typer.echo(tabulate(table, headers=["Id", "Title"]))

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

        campaigns = await client.campaigns(campaign_group_id)

        ideas = []
        with client.request_progress_observer.progress:
            ideas = await asyncio.gather(*[client.campaign_ideas(c.id) for c in campaigns])
            ideas = [i for campaign_ideas in ideas for i in campaign_ideas]
        typer.echo()

        table = [[i.id, i.title] for i in ideas]
        table.sort(key=lambda i: i[0])

        typer.echo(tabulate(table, headers=["Id", "Title"]))

    asyncio.run(inner())
