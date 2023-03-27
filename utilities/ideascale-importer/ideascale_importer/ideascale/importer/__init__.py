import asyncio
import asyncpg
import csv
import rich.panel
import rich.prompt
import rich.table
import rich.console
from typing import Dict, List, Optional


from . import config, db_mapper
from ideascale_importer.ideascale.client import CampaignGroup, Client, Funnel
import ideascale_importer.db


class ReadConfigException(Exception):
    ...


class ReadProposalsScoresCsv(Exception):
    ...


class Importer:
    def __init__(
        self,
        api_token: str,
        database_url: str,
        config_path: Optional[str],
        event_id: Optional[int],
        campaign_group_id: Optional[int],
        stage_id: Optional[int],
        proposals_scores_csv_path: Optional[str],
    ):
        self.api_token = api_token
        self.database_url = database_url
        self.event_id = event_id
        self.campaign_group_id = campaign_group_id
        self.stage_id = stage_id
        self.conn: asyncpg.Connection | None = None

        try:
            self.config = config.from_json_file(config_path or "ideascale-importer-config.json")
        except Exception as e:
            raise ReadConfigException(repr(e)) from e

        self.proposals_impact_scores: Dict[int, int] = {}
        if proposals_scores_csv_path is not None:
            try:
                with open(proposals_scores_csv_path) as f:
                    r = csv.DictReader(f)
                    for row in r:
                        proposal_id = int(row[self.config.proposals_scores_csv.id_field], base=10)

                        # Multiply the scores by 100 so we have 3.14 -> 314 which is
                        # the format app expects.
                        score = int(float(row[self.config.proposals_scores_csv.score_field])*100)

                        self.proposals_impact_scores[proposal_id] = score
            except Exception as e:
                raise ReadProposalsScoresCsv(repr(e)) from e

    async def connect(self):
        if self.conn is None:
            self.conn = await ideascale_importer.db.connect(self.database_url)

    async def close(self):
        if self.conn is not None:
            await self.conn.close()

    async def import_all(self):
        if self.conn is None:
            raise Exception("Not connected to the database")

        console = rich.console.Console()

        if self.event_id is None:
            select_event_id = rich.prompt.Prompt.ask("Enter the event database id")
            event_id = int(select_event_id, base=10)
        else:
            event_id = self.event_id

        if not await ideascale_importer.db.event_exists(self.conn, event_id):
            console.print("\n[red]No event exists with the given id[/red]")
            return

        client = Client(self.api_token)

        groups = []
        with client.inner.request_progress_observer:
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

        group: Optional[CampaignGroup] = None
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

            funnels: List[Funnel] = []
            with client.inner.request_progress_observer:
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
        with client.inner.request_progress_observer:
            ideas = await client.stage_ideas(stage_id)

        vote_options_id = await ideascale_importer.db.get_vote_options_id(self.conn, "yes,no")
        mapper = db_mapper.Mapper(vote_options_id, self.config)

        challenges = [mapper.map_challenge(a, event_id) for a in group.campaigns]
        challenge_count = len(challenges)
        proposal_count = 0

        async with self.conn.transaction():
            await ideascale_importer.db.insert_many(self.conn, challenges)

            proposals = [
                mapper.map_proposal(a, self.proposals_impact_scores)
                for a in ideas
            ]
            proposal_count = len(proposals)
            await ideascale_importer.db.insert_many(self.conn, proposals)

        console.print(f"Imported {challenge_count} challenges and {proposal_count} proposals")
