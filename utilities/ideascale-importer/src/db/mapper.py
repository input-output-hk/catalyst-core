import dataclasses
from markdownify import markdownify
import re
from typing import Mapping

import db.models
import ideascale


class Mapper:
    def map_challenge(self, a: ideascale.Campaign, election_id: int) -> db.models.Challenge:
        reward = parse_reward(a.tagline)

        return db.models.Challenge(
            id=a.id,
            election=election_id,
            category=map_challenge_category(a),
            title=a.name,
            description=html_to_md(a.description),
            rewards_currency=reward.currency,
            rewards_total=reward.amount,
            proposers_rewards=reward.amount,
            vote_options=None,  # TODO: Should get this id from the DB I guess
            extra=None          # TODO: Not sure if we have this information in IdeaScale
        )

    def map_proposal(self, a: ideascale.Idea, challenge_id_to_row_id_map: Mapping[int, int]) -> db.models.Proposal:
        return db.models.Proposal(
            id=a.id,
            challenge=challenge_id_to_row_id_map[a.campaign_id],
            title=html_to_md(a.title),
            summary=html_to_md(a.text),
            public_key="",
            funds=0,
            url="",
            files_url="",
            impact_score=0,
            extra={},
            proposer_name=a.author_info.name,
            proposer_contact="",
            proposer_relevant_experience="",
            proposer_url="",
            bb_proposal_id=None,
            bb_vote_options="yes,no",
        )


def html_to_md(s: str) -> str:
    tags_to_strip = ['a', 'b', 'img', 'strong', 'u', 'i', 'embed', 'iframe']
    return markdownify(s, strip=tags_to_strip).strip()


@dataclasses.dataclass
class Reward:
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


def map_challenge_category(c: ideascale.Campaign) -> str:
    r = c.name.lower()

    if 'catalyst natives' in r:
        return 'native'
    elif 'challenge setting' in r:
        return 'community-choice'
    else:
        return 'simple'
