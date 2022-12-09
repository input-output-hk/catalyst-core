import asyncio
import rich.panel
import rich.prompt
import rich.table
import rich.console
import typer

import ideascale

app = typer.Typer()

@app.command()
def import_challenges(
    api_token: str = typer.Option(..., help="IdeaScale API token"),
):
    """
    Import challenges from IdeaScale (a Challenge is represented as a Campaign in IdeaScale)
    """
    async def inner():
        client = ideascale.client_with_progress(api_token)

        groups = []
        with client.request_progress_observer:
            groups = [g for g in await client.campaign_groups() if g.name.lower().startswith("fund")]

        if len(groups) > 0:
            typer.echo()
            funds_table = rich.table.Table("Id", "Name", title="Available Funds")

            for group in groups:
                funds_table.add_row(str(group.id), group.name)

            rich.console.Console().print(funds_table)

            campaign_group_id = rich.prompt.Prompt.ask("Enter a fund id", choices=list(map(lambda g: str(g.id), groups)), show_choices=False)
            typer.echo()

            with client.request_progress_observer:
                ideas = await client.campaign_group_ideas(int(campaign_group_id, base=10))

    asyncio.run(inner())
