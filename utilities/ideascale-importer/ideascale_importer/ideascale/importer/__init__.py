import asyncpg
import csv
from loguru import logger
from typing import Dict, Optional


from . import config, db_mapper
from ideascale_importer.ideascale.client import CampaignGroup, Client
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
        event_id: int,
        campaign_group_id: int,
        stage_id: int,
        proposals_scores_csv_path: Optional[str],
        ideascale_api_url: str,
    ):
        self.api_token = api_token
        self.database_url = database_url
        self.event_id = event_id
        self.campaign_group_id = campaign_group_id
        self.stage_id = stage_id
        self.conn: asyncpg.Connection | None = None
        self.ideascale_api_url = ideascale_api_url

        try:
            config_file_path = config_path or "ideascale-importer-config.json"
            logger.debug("Reading configuration file", path=config_file_path)
            self.config = config.from_json_file(config_file_path)
        except Exception as e:
            raise ReadConfigException(repr(e)) from e

        self.proposals_impact_scores: Dict[int, int] = {}
        if proposals_scores_csv_path is not None:
            logger.debug("Reading proposals impact scores from file", path=proposals_scores_csv_path)

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
            logger.info("Connecting to the database")
            self.conn = await ideascale_importer.db.connect(self.database_url)

    async def close(self):
        if self.conn is not None:
            await self.conn.close()

    async def import_all(self):
        if self.conn is None:
            raise Exception("Not connected to the database")

        if not await ideascale_importer.db.event_exists(self.conn, self.event_id):
            logger.error("No event exists with the given id")
            return

        client = Client(self.api_token, self.ideascale_api_url)

        groups = [g for g in await client.campaign_groups() if g.name.lower().startswith("fund")]

        if len(groups) == 0:
            logger.warning("No funds found")
            return

        group: Optional[CampaignGroup] = None
        for g in groups:
            if g.id == self.campaign_group_id:
                group = g
                break

        if group is None:
            logger.error("Campaign group id does not correspond to any fund campaign group id")
            return

        ideas = await client.stage_ideas(self.stage_id)

        vote_options_id = await ideascale_importer.db.get_vote_options_id(self.conn, "yes,no")
        mapper = db_mapper.Mapper(vote_options_id, self.config)

        challenges = [mapper.map_challenge(a, self.event_id) for a in group.campaigns]
        challenge_count = len(challenges)
        proposal_count = 0

        async with self.conn.transaction():
            await ideascale_importer.db.upsert_many(self.conn, challenges, conflict_cols=["id"])

            proposals = [
                mapper.map_proposal(a, self.proposals_impact_scores)
                for a in ideas
            ]
            proposal_count = len(proposals)
            await ideascale_importer.db.upsert_many(self.conn, proposals, conflict_cols=["id"])

        logger.info(
            "Imported challenges and proposals",
            challenge_count=challenge_count, proposal_count=proposal_count)
