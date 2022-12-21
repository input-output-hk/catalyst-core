import dataclasses
from typing import Any, Mapping, Optional


class Model:
    @staticmethod
    def table() -> str:
        raise NotImplementedError()


@dataclasses.dataclass
class Challenge(Model):
    id: int
    election: int
    category: str
    title: str
    description: str
    rewards_currency: Optional[str]
    rewards_total: Optional[int]
    proposers_rewards: Optional[int]
    vote_options: Optional[int]
    extra: Optional[Mapping[str, Any]]

    @staticmethod
    def table() -> str:
        return "challenge"


@dataclasses.dataclass
class Proposal(Model):
    id: int
    challenge: int
    title: str
    summary: str
    public_key: str
    funds: int
    url: str
    files_url: str
    impact_score: Optional[float]

    extra: Optional[Mapping[str, str]]

    proposer_name: str
    proposer_contact: str
    proposer_url: str
    proposer_relevant_experience: str

    bb_proposal_id: Optional[bytes]
    bb_vote_options: Optional[str]

    @staticmethod
    def table() -> str:
        return "proposal"
