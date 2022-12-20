import asyncio
import asyncpg
import json
import rich.panel
import rich.prompt
import rich.table
import rich.console
from typing import List, Optional

import config
import db
import db.mapper
import ideascale


class ReadConfigException(Exception):
    ...


class Importer:
    def __init__(
        self,
        api_token: str,
        database_url: str,
        election_id: Optional[int],
        campaign_group_id: Optional[int],
        stage_id: Optional[int],
        config_path: Optional[str],
    ):
        self.api_token = api_token
        self.database_url = database_url
        self.election_id = election_id
        self.campaign_group_id = campaign_group_id
        self.stage_id = stage_id
        self.conn: asyncpg.Connection | None = None

        try:
            self.config = config.load(config_path or "config.json")
        except Exception as e:
            raise ReadConfigException(e)

    async def connect(self):
        if self.conn is None:
            self.conn = await asyncpg.connect(self.database_url)
            await self.conn.set_type_codec("jsonb", encoder=json.dumps, decoder=json.loads, schema="pg_catalog")

    async def close(self):
        if self.conn is not None:
            await self.conn.close()

    async def import_all(self):
        if self.conn is None:
            raise Exception("Not connected to the database")

        console = rich.console.Console()

        if self.election_id is None:
            select_election_id = rich.prompt.Prompt.ask("Enter the election database id")
            election_id = int(select_election_id, base=10)
        else:
            election_id = self.election_id

        if not await db.election_exists(self.conn, election_id):
            console.print("\n[red]No election exists with the given id[/red]")
            return

        client = ideascale.client_with_progress(self.api_token)

        groups = []
        with client.request_progress_observer:
            groups = [g for g in await client.campaign_groups() if g.name.lower().startswith("fund")]

        if len(groups) == 0:
            console.print("No funds found")
            return

        if self.campaign_group_id is None:
            console.print()
            funds_table = rich.table.Table("Id", "Name", title="Available Funds")

            for g in groups:
                funds_table.add_row(str(g.id), g.name)
            console.print(funds_table)

            selected_campaign_group_id = rich.prompt.Prompt.ask(
                "Select a fund id",
                choices=list(map(lambda g: str(g.id), groups)),
                show_choices=False)
            campaign_group_id = int(selected_campaign_group_id, base=10)
            console.print()
        else:
            campaign_group_id = self.campaign_group_id

        group: Optional[ideascale.CampaignGroup] = None
        for g in groups:
            if g.id == campaign_group_id:
                group = g
                break

        if group is None:
            console.print("\n[red]Campaign group id does not correspond to any fund campaign group id[/red]")
            return

        if self.stage_id is None:
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
        else:
            stage_id = self.stage_id

        ideas = []
        with client.request_progress_observer:
            ideas = await client.stage_ideas(stage_id)

        vote_options_id = await db.get_vote_options_id(self.conn, "yes,no")
        mapper = db.mapper.Mapper(vote_options_id, self.config)

        challenges = [mapper.map_challenge(a, election_id) for a in group.campaigns]
        challenge_count = len(challenges)
        proposal_count = 0

        async with self.conn.transaction():
            challenge_row_ids = await db.insert_many(self.conn, challenges, returning="row_id")

            # This mapping is needed because the 'challenge' foreign key in the 'proposal' table
            # expects challenge.row_id which is generated by Postgres and
            # not challenge.id which comes from IdeaScale.
            challenge_id_to_row_id_map = {}
            for challenge, row_id in zip(challenges, challenge_row_ids):
                challenge_id_to_row_id_map[challenge.id] = row_id

            proposals = [mapper.map_proposal(a, challenge_id_to_row_id_map) for a in ideas]
            proposal_count = len(proposals)
            await db.insert_many(self.conn, proposals)

        console.print(f"Imported {challenge_count} challenges and {proposal_count} proposals")
