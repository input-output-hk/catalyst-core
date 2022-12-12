import asyncio
import psycopg
import rich.panel
import rich.prompt
import rich.table
import rich.console
import typer
from typing import List

import db
import ideascale

app = typer.Typer()


@app.command()
def import_all(
    api_token: str = typer.Option(..., help="IdeaScale API token"),
    database_url: str = typer.Option(..., help="Postgres database URL"),
):
    """
    Import all fund data from IdeaScale
    """
    async def inner():
        console = rich.console.Console()

        db_conn = await psycopg.AsyncConnection.connect(database_url)

        select_election_id = rich.prompt.Prompt.ask("Enter the election database id")
        if not await db.election_exists(db_conn, int(select_election_id, base=10)):
            console.print("\n[red]No election exists with the given id[/red]")
            return

        client = ideascale.client_with_progress(api_token)

        groups = []
        with client.request_progress_observer:
            groups = [g for g in await client.campaign_groups() if g.name.lower().startswith("fund")]

        if len(groups) == 0:
            console.print("No funds found")
            return

        console.print()
        funds_table = rich.table.Table("Id", "Name", title="Available Funds")

        for group in groups:
            funds_table.add_row(str(group.id), group.name)

        console.print(funds_table)

        selected_campaign_group_id = rich.prompt.Prompt.ask(
            "Select a fund id",
            choices=list(map(lambda g: str(g.id), groups)),
            show_choices=False)
        campaign_group_id = int(selected_campaign_group_id, base=10)
        console.print()

        # Garanteed to match only 1
        group = [g for g in groups if g.id == campaign_group_id][0]

        funnel_ids = set()
        for c in group.campaigns:
            if c.funnel_id is not None:
                funnel_ids.add(c.funnel_id)

        funnels: List[ideascale.Funnel] = []
        with client.request_progress_observer:
            funnels = await asyncio.gather(*[client.funnel(id) for id in funnel_ids])

        stages = [stage for funnel in funnels for stage in funnel.stages]

        if len(stages) == 0:
            console.print("No stages found")
            return

        stages_table = rich.table.Table("Id", "Label", "Funnel Name", title="Available Stages")

        stages.sort(key=lambda s: f"{s.funnel_name}{s.id}")
        for stage in stages:
            stages_table.add_row(str(stage.id), stage.label, stage.funnel_name)
        console.print(stages_table)

        selected_stage_id = rich.prompt.Prompt.ask(
            "Select a stage id",
            choices=list(map(lambda s: str(s.id), stages)),
            show_choices=False)
        stage_id = int(selected_stage_id, base=10)
        console.print()

        ideas = []
        with client.request_progress_observer:
            ideas = await client.stage_ideas(stage_id)
        console.print(f"Fetched {len(ideas)} ideas")

        console.print("SHOULD MAP AND INSERT DATA INTO POSTGRES TABLES NOW")

        await db_conn.close()

    asyncio.run(inner())
