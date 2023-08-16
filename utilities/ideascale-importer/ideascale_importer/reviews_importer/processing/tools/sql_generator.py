"""Module to postprocess reviews."""
from typing import List

from ..db import models
from .. import utils
from .importer import Importer, IdeascaleImporter


class SQLGenerator:
    """Module to prepare SQL files for moderation tool."""

    def __init__(self, importer: Importer, ideascale: IdeascaleImporter, path: str):
        """Initialize entities."""
        self.importer = importer
        self.ideascale = ideascale
        self.path = path

    def pg_esc(self, line: str | None) -> str | None:
        """Escape a string for postgres."""
        if line is None:
            return None
        return line.replace("'", "''")

    def funds(self, event_id: int):
        """Generate SQL for funds."""

        events = f"""
(
    {event_id},
    {event_id}, -- event id
    'Catalyst Fund {event_id}', -- title
    0,
    0
)
"""

        data = f"""--sql
    -- Fund {event_id}
    INSERT INTO event
    (
        row_id, name, description, committee_size, committee_threshold
    )
    VALUES
    {events}
    ;

    """
        self.write("funds.sql", data)

    async def challenges(self, event_id: int, group_id: int):
        """Generate SQL for challenges."""
        await self.ideascale.import_challenges(group_id)

        objectives = ""

        for challenge in self.ideascale.challenges:
            id = challenge.id
            challenge_type = "simple"
            title = self.pg_esc(challenge.title)
            description = ""
            rewards_total = 0
            proposers_rewards = 0

            if len(objectives) > 0:
                objectives += ",\n"

            objectives += f"""
(
    {id}, -- Objective ID
    {id}, -- Objective ID
    {event_id}, -- event id
    'catalyst-{challenge_type}', -- category
    '{title}', -- title
    '{description}', -- description
    'ADA', -- Currency
    {rewards_total}, -- rewards total
    NULL, -- rewards_total_lovelace
    {proposers_rewards}, -- proposers rewards
    1, -- vote_options
    '{{}}' -- extra objective data
)
"""

        data = f"""--sql
    -- Challenges for Fund {event_id}
    INSERT INTO objective
    (
        row_id,
        id,
        event,
        category,
        title,
        description,
        rewards_currency,
        rewards_total,
        rewards_total_lovelace,
        proposers_rewards,
        vote_options,
        extra)
    VALUES
    {objectives}
    ;

    """
        self.write("challenges.sql", data)

    async def proposals(self, event_id: int, stage_id: int):
        """Generate SQL for proposals."""
        await self.ideascale.import_proposals(stage_id=stage_id)

        proposals = ""

        for proposal in self.ideascale.proposals:
            proposal_id = proposal.id
            challenge_id = proposal.challenge_id
            proposal_title = self.pg_esc(proposal.title)
            proposal_summary = ""
            category = proposal.challenge_id
            proposal_public_key = ""
            proposal_funds = 0
            proposal_url = proposal.url
            proposal_files_url = ""
            proposal_impact_score = 0

            proposer_name = ""
            proposer_contact = ""
            proposer_url = ""
            proposer_relevant_experience = ""

            if len(proposals) > 0:
                proposals += ",\n"

            proposals += f"""
(
    {proposal_id},  -- id
    {challenge_id}, -- objective
    '{proposal_title}',  -- title
    '{proposal_summary}',  -- summary
    {category}, -- category - VITSS Compat ONLY
    '{proposal_public_key}', -- Public Payment Key
    '{proposal_funds}', -- funds
    '{proposal_url}', -- url
    '{proposal_files_url}', -- files_url
    {proposal_impact_score}, -- impact_score
    '{proposer_name}', -- proposer name
    '{proposer_contact}', -- proposer contact
    '{proposer_url}', -- proposer URL
    '{proposer_relevant_experience}' -- relevant experience
)
"""

        data = f"""--sql
-- All Proposals for  FUND {event_id}
INSERT INTO proposal
(
    id,
    objective,
    title,
    summary,
    category,
    public_key,
    funds,
    url,
    files_url,
    impact_score,
    proposer_name,
    proposer_contact,
    proposer_url,
    proposer_relevant_experience
)
VALUES
{proposals}
;
    """
        self.write("proposals.sql", data)

    def reviews(self, path: str, event_id: int):
        self.importer.load_reviews(path, event_id)

        data = f"""
--sql
-- All reviewers levels for FUND {event_id}
INSERT INTO reviewer_level
(
    row_id,
    name,
    total_reward_pct,
    event
)
VALUES
(0, 'LV0', 20, {event_id}),
(1, 'LV1', 80, {event_id});
"""

        reviews = ""
        for review in self.importer.reviews:
            row_id = review.id
            proposal_id = review.proposal_id
            assessor = review.assessor
            assessor_level = review.level
            impact_alignment_rating_given = review.impact_rating
            impact_alignment_note = self.pg_esc(review.impact_note)
            feasibility_rating_given = review.feasibility_rating
            feasibility_note = self.pg_esc(review.feasibility_note)
            auditability_rating_given = review.auditability_rating
            auditability_note = self.pg_esc(review.auditability_note)

            if len(reviews) > 0:
                reviews += ",\n"

            reviews += f"""
(
    {row_id},  -- id
    {proposal_id}, -- proposal_id
    '{assessor}',  -- assessor
    '{assessor_level}',  -- assessor_level
    {impact_alignment_rating_given},
    '{impact_alignment_note}',
    {feasibility_rating_given},
    '{feasibility_note}',
    {auditability_rating_given},
    '{auditability_note}'
   
)
"""

        data += f"""--sql
-- All ProposalsReviews for FUND {event_id}
INSERT INTO proposal_review
(
    row_id,
    proposal_id,
    assessor,
    assessor_level,
    impact_alignment_rating_given,
    impact_alignment_note,
    feasibility_rating_given,
    feasibility_note,
    auditability_rating_given,
    auditability_note
)
VALUES
{reviews}
;
    """
        self.write("reviews.sql", data)

    def write(self, name: str, data: str):
        f = open(f"{self.path}/{name}", "w")
        f.write(data)
        f.close()
