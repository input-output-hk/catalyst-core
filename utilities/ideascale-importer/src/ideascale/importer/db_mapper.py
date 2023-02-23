import dataclasses
from markdownify import markdownify
import re
from typing import Any, Mapping

from . import config
import db
import db.models
import ideascale.client


class Mapper:
    """
    Holds configuration and executes mapping functions.
    """

    def __init__(self, vote_options_id: int, config: config.Config):
        self.config = config
        self.vote_options_id = vote_options_id

    def map_challenge(self, a: ideascale.client.Campaign, election_id: int) -> db.models.Challenge:
        """
        Maps a IdeaScale campaign into a challenge.
        """

        reward = parse_reward(a.tagline)

        return db.models.Challenge(
            id=a.id,
            election=election_id,
            category=get_challenge_category(a),
            title=a.name,
            description=html_to_md(a.description),
            rewards_currency=reward.currency,
            rewards_total=reward.amount,
            proposers_rewards=reward.amount,
            vote_options=self.vote_options_id,
            extra={"url": {"challenge": a.campaign_url}}
        )

    def map_proposal(
        self,
        a: ideascale.client.Idea,
        challenge_id_to_row_id_map: Mapping[int, int],
        impact_scores: Mapping[int, int],
    ) -> db.models.Proposal:
        """
        Maps an IdeaScale idea into a proposal.
        """

        field_mappings = self.config.proposals.field_mappings

        proposer_name = ", ".join([a.author_info.name]+a.contributors_name())
        proposer_url = get_value(a.custom_fields_by_key, field_mappings.proposer_url) or ""
        proposer_relevant_experience = html_to_md(get_value(
            a.custom_fields_by_key,
            field_mappings.proposer_relevant_experience
        ) or "")
        funds = int(get_value(a.custom_fields_by_key, field_mappings.funds) or "0", base=10)
        public_key = get_value(a.custom_fields_by_key, field_mappings.public_key) or ""

        extra_fields_mappings = self.config.proposals.extra_field_mappings

        extra = {}
        for k, v in extra_fields_mappings.items():
            mv = get_value(a.custom_fields_by_key, v)
            if mv is not None:
                extra[k] = html_to_md(mv)

        return db.models.Proposal(
            id=a.id,
            challenge=challenge_id_to_row_id_map[a.campaign_id],
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
            bb_vote_options="yes,no",
        )


def get_value(m: Mapping[str, Any], f: config.FieldMapping) -> Any | None:
    """
    Gets the value of the given mapping key in the given mapping.
    """

    if isinstance(f, list):
        for k in f:
            if k in m:
                return m[k]
    else:
        if f in m:
            return m[f]
    return None


def html_to_md(s: str) -> str:
    """
    Transforms a HTML string into a Markdown string.
    """

    tags_to_strip = ['a', 'b', 'img', 'strong', 'u', 'i', 'embed', 'iframe']
    return markdownify(s, strip=tags_to_strip).strip()


@dataclasses.dataclass
class Reward:
    """
    Represents a reward.
    """

    amount: int
    currency: str


class InvalidRewardsString(Exception):
    ...


def parse_reward(s: str) -> Reward:
    """
    Parses budget and currency from 3 different templates:
        1. $500,000 in ada
        2. $200,000 in CLAP tokens
        3. 12,800,000 ada
    """
    result = re.search(r"\$?(.*?)\s+(?:in\s)?(\S*)", s)
    if result is None:
        raise InvalidRewardsString()

    amount = re.sub(r"\D", "", result.group(1))
    currency = result.group(2)
    return Reward(amount=int(amount, base=10), currency=currency.upper())


def get_challenge_category(c: ideascale.client.Campaign) -> str:
    """
    Computes the challenge category of a given campaign.
    """

    r = c.name.lower()

    if 'catalyst natives' in r:
        return 'native'
    elif 'challenge setting' in r:
        return 'community-choice'
    else:
        return 'simple'
