from typing import Optional
from ideascale_importer.db.models import Objective, Proposal
from pydantic import BaseModel


class ProposalJson(BaseModel):
    """A proposal in JSON used for output artifacts."""

    category_name: str
    chain_vote_options: str
    challenge_id: str
    challenge_type: str
    chain_vote_type: str
    internal_id: str
    proposal_funds: str
    proposal_id: str
    proposal_impact_score: str
    proposal_summary: str
    proposal_title: str
    proposal_url: str
    proposer_email: Optional[str] = None
    proposer_name: Optional[str] = None
    proposer_relevant_experience: Optional[str] = None
    proposer_url: Optional[str] = None
    proposal_solution: Optional[str] = None
    files_url: str


class ChallengesJson(BaseModel):
    id: str
    internal_id: int
    title: str
    challenge_type: str
    challenge_url: str
    description: str
    fund_id: str
    rewards_total: str
    proposers_rewards: str


def objective_to_challenge_json(obj: Objective, ideascale_url: str, idx: int = 0) -> ChallengesJson:
    c_url = f"{ideascale_url}/c/campaigns/{obj.id}/"
    return ChallengesJson.model_validate(
        {
            "id": f"{idx}",
            "internal_id": obj.id,
            "title": obj.title,
            "challenge_type": obj.category.removeprefix("catalyst-"),
            "challenge_url": c_url,
            "description": obj.description,
            "fund_id": f"{obj.event}",
            "rewards_total": f"{obj.rewards_total}",
            "proposers_rewards": f"{obj.proposers_rewards}",
        }
    )


def json_from_proposal(prop: Proposal, challenge: ChallengesJson, fund_id: int, idx: int = 0) -> ProposalJson:
    if prop.proposer_relevant_experience == "":
        experience = None
    else:
        experience = prop.proposer_relevant_experience
    if prop.extra is not None:
        solution = prop.extra.get("solution", None)
    else:
        solution = None
    return ProposalJson.model_validate(
        {
            "category_name": f"Fund {fund_id}",
            "chain_vote_options": "blank,yes,no",
            "challenge_id": challenge.id,
            "challenge_type": challenge.challenge_type,
            "chain_vote_type": "private",
            "internal_id": f"{idx}",
            "proposal_funds": f"{prop.funds}",
            "proposal_id": f"{prop.id}",
            "proposal_impact_score": f"{prop.impact_score}",
            "proposal_summary": prop.summary,
            "proposal_title": prop.title,
            "proposal_url": prop.url,
            "proposer_name": prop.proposer_name,
            "proposer_relevant_experience": experience,
            "proposal_solution": solution,
            "files_url": prop.files_url,
        }
    )


class FundsJson(BaseModel):
    id: int
    goal: str
    threshold: int
    rewards_info: str
