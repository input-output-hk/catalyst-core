from typing import Mapping

import db.models
import ideascale


def map_challenge(a: ideascale.Campaign, election_id: int) -> db.models.Challenge:
    return db.models.Challenge(
        id=a.id,
        election=election_id,
        category="simple",
        title=a.name,
        description=a.description,
        rewards_currency=None,
        rewards_total=None
    )


def map_proposal(a: ideascale.Idea, challenge_id_to_row_id_map: Mapping[int, int]) -> db.models.Proposal:
    return db.models.Proposal(
        id=a.id,
        challenge=challenge_id_to_row_id_map[a.campaign_id],
        title=a.title,
        summary=a.text,
        public_key="what",
        funds=0,
        url="url",
        files_url="files_url",
        impact_score=0,
        extra="{}",
        proposer_name="proposer_name",
        proposer_contact="",
        proposer_relevant_experience=",",
        proposer_url="",
        bb_proposal_id=None,
        bb_vote_options=None,
    )
