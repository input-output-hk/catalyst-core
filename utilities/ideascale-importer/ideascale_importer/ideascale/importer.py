"""Ideascale importer."""

import re
import asyncpg
import csv
from dataclasses import dataclass
import json
from loguru import logger
from markdownify import markdownify
import pydantic
from typing import Any, Dict, List, Mapping, Optional, Union

from .client import Campaign, CampaignGroup, Client, Idea
import ideascale_importer.db


FieldMapping = Union[str, List[str]]


@dataclass
class ProposalsFieldsMappingConfig:
    """Represents the available configuration fields used in proposal fields mapping."""

    proposer_url: FieldMapping
    proposer_relevant_experience: FieldMapping
    funds: FieldMapping
    public_key: FieldMapping


@dataclass
class ProposalsConfig:
    """Represents the available configuration fields used in proposal processing."""

    field_mappings: ProposalsFieldsMappingConfig
    extra_field_mappings: Mapping[str, FieldMapping]  # noqa: F821


@dataclass
class ProposalsScoresCsvConfig:
    """Represents the available configuration fields for proposal scores from the CSV file."""

    id_field: str
    score_field: str


@dataclass
class Config:
    """Represents the available configuration fields."""

    proposals: ProposalsConfig
    proposals_scores_csv: ProposalsScoresCsvConfig

    @staticmethod
    def from_json_file(path: str) -> "Config":
        """Load configuration from a JSON file."""
        with open(path) as f:
            return pydantic.tools.parse_obj_as(Config, json.load(f))


class ReadConfigException(Exception):
    """Raised when the configuration file cannot be read."""

    def __init__(self, cause: str):
        super().__init__(f"Failed to read config file: {cause}")


class ReadProposalsScoresCsv(Exception):
    """Raised when the proposals impact scores csv cannot be read."""

    def __init__(self, cause: str):
        super().__init__(f"Failed to read proposals impact score file: {cause}")


class MapObjectiveError(Exception):
    """Raised when mapping an objective from campaign data fails."""

    def __init__(self, objective_field: str, campaign_field: str, cause: str):
        super().__init__(f"Failed to map objective '{objective_field}' from campaign '{campaign_field}': {cause}")


class Mapper:
    """Holds configuration and executes mapping functions."""

    def __init__(self, vote_options_id: int, config: Config):
        """Initialize the mapper."""
        self.config = config
        self.vote_options_id = vote_options_id

    def map_objective(self, a: Campaign, event_id: int) -> ideascale_importer.db.models.Challenge:
        """Map a IdeaScale campaign into a objective."""
        try:
            reward = parse_reward(a.tagline)
        except InvalidRewardsString as e:
            raise MapObjectiveError("reward", "tagline", str(e))

        return ideascale_importer.db.models.Challenge(
            row_id=0,
            id=a.id,
            event=event_id,
            category=get_objective_category(a),
            title=a.name,
            description=html_to_md(a.description),
            rewards_currency=reward.currency,
            rewards_total=reward.amount,
            proposers_rewards=reward.amount,
            vote_options=self.vote_options_id,
            extra={"url": {"objective": a.campaign_url}},
        )

    def map_proposal(
        self,
        a: Idea,
        impact_scores: Mapping[int, int],
    ) -> ideascale_importer.db.models.Proposal:
        """Map an IdeaScale idea into a proposal."""
        field_mappings = self.config.proposals.field_mappings

        proposer_name = ", ".join([a.author_info.name] + a.contributors_name())
        proposer_url = get_value(a.custom_fields_by_key, field_mappings.proposer_url) or ""
        proposer_relevant_experience = html_to_md(
            get_value(a.custom_fields_by_key, field_mappings.proposer_relevant_experience) or ""
        )
        funds = int(get_value(a.custom_fields_by_key, field_mappings.funds) or "0", base=10)
        public_key = get_value(a.custom_fields_by_key, field_mappings.public_key) or ""

        extra_fields_mappings = self.config.proposals.extra_field_mappings

        extra = {}
        for k, v in extra_fields_mappings.items():
            mv = get_value(a.custom_fields_by_key, v)
            if mv is not None:
                extra[k] = html_to_md(mv)

        return ideascale_importer.db.models.Proposal(
            id=a.id,
            objective=0,  # should be set later
            title=html_to_md(a.title),
            summary=html_to_md(a.text),
            category="",
            public_key=public_key,
            funds=funds,
            url=a.url,
            files_url="",
            impact_score=impact_scores.get(a.id, 0),
            extra=extra,
            proposer_name=proposer_name,
            proposer_contact="",
            proposer_relevant_experience=proposer_relevant_experience,
            proposer_url=proposer_url,
            bb_proposal_id=None,
            bb_vote_options=["yes", "no"],
        )


def get_value(m: Mapping[str, Any], f: FieldMapping) -> Any | None:
    """Get the value of the given mapping key in the given mapping."""
    if isinstance(f, list):
        for k in f:
            if k in m:
                return m[k]
    else:
        if f in m:
            return m[f]
    return None


def html_to_md(s: str) -> str:
    """Transform a HTML string into a Markdown string."""
    tags_to_strip = ["a", "b", "img", "strong", "u", "i", "embed", "iframe"]
    return markdownify(s, strip=tags_to_strip).strip()


@dataclass
class Reward:
    """Represents a reward."""

    amount: int
    currency: str


class InvalidRewardsString(Exception):
    """Raised when the reward string cannot be parsed."""

    def __init__(self):
        super().__init__("Invalid rewards string")


def parse_reward(s: str) -> Reward:
    """Parse budget and currency from 3 different templates.

    1. $500,000 in ada
    2. $200,000 in CLAP tokens
    3. 12,800,000 ada.
    """
    result = re.search(r"\$?(.*?)\s+(?:in\s)?(\S*)", s)
    if result is None:
        raise InvalidRewardsString()

    amount = re.sub(r"\D", "", result.group(1))
    currency = result.group(2)
    return Reward(amount=int(amount, base=10), currency=currency.upper())


def get_objective_category(c: Campaign) -> str:
    """Compute the objective category of a given campaign."""
    r = c.name.lower()

    if "catalyst natives" in r:
        return "catalyst-native"
    elif "objective setting" in r:
        return "catalyst-community-choice"
    else:
        return "catalyst-simple"


class Importer:
    """Ideascale importer."""

    def __init__(
        self,
        api_token: str,
        database_url: str,
        config_path: Optional[str],
        event_id: int,
        campaign_group_id: int,
        stage_ids: [int],
        proposals_scores_csv_path: Optional[str],
        ideascale_api_url: str,
    ):
        """Initialize the importer."""
        self.api_token = api_token
        self.database_url = database_url
        self.event_id = event_id
        self.campaign_group_id = campaign_group_id
        self.stage_ids = stage_ids
        self.conn: asyncpg.Connection | None = None
        self.ideascale_api_url = ideascale_api_url

        try:
            config_file_path = config_path or "ideascale-importer-config.json"
            logger.debug("Reading configuration file", path=config_file_path)
            self.config = Config.from_json_file(config_file_path)
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
                        score = int(float(row[self.config.proposals_scores_csv.score_field]) * 100)

                        self.proposals_impact_scores[proposal_id] = score
            except Exception as e:
                raise ReadProposalsScoresCsv(repr(e)) from e

    async def connect(self, *args, **kwargs):
        """Connect to the database."""
        if self.conn is None:
            logger.info("Connecting to the database")
            self.conn = await ideascale_importer.db.connect(self.database_url, *args, **kwargs)

    async def close(self):
        """Close the connection to the database."""
        if self.conn is not None:
            await self.conn.close()

    async def run(self):
        """Run the importer."""
        if self.conn is None:
            raise Exception("Not connected to the database")

        if not await ideascale_importer.db.event_exists(self.conn, self.event_id):
            logger.error("No event exists with the given id")
            return

        client = Client(self.api_token, self.ideascale_api_url)

        groups = await client.campaign_groups()
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

        ideas = []
        for stage_id in self.stage_ids:
            ideas.extend(await client.stage_ideas(stage_id=stage_id))

        vote_options_id = await ideascale_importer.db.get_vote_options_id(self.conn, ["yes", "no"])
        mapper = Mapper(vote_options_id, self.config)

        objectives = [mapper.map_objective(a, self.event_id) for a in group.campaigns]
        objective_count = len(objectives)
        proposal_count = 0

        async with self.conn.transaction():
            inserted_objectives = await ideascale_importer.db.upsert_many(self.conn, objectives, conflict_cols=["id", "event"])
            inserted_objectives_ix = {o.id: o for o in inserted_objectives}

            proposals_with_campaign_id = [(a.campaign_id, mapper.map_proposal(a, self.proposals_impact_scores)) for a in ideas]
            proposals = []
            for objective_id, p in proposals_with_campaign_id:
                if objective_id in inserted_objectives_ix:
                    p.objective = inserted_objectives_ix[objective_id].row_id
                    proposals.append(p)

            proposal_count = len(proposals)
            await ideascale_importer.db.upsert_many(self.conn, proposals, conflict_cols=["id", "objective"])

        logger.info("Imported objectives and proposals", objective_count=objective_count, proposal_count=proposal_count)
