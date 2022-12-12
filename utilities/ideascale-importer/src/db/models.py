import dataclasses
from typing import Optional


class Model:
    @staticmethod
    def table() -> str:
        raise NotImplementedError()


@dataclasses.dataclass
class Challenge(Model):
    id: int
    election: int
    type: str
    title: str
    description: str
    rewards_currency: str
    rewards_total: int

    @staticmethod
    def table() -> str:
        return "challenges"


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
    impact_score: int

    extra: Optional[str]  # json

    proposer_name: str
    proposer_contact: str
    proposer_url: str
    proposer_relevant_experience: str

    bb_proposal_id: bytes
    bb_vote_options: str

    @staticmethod
    def table() -> str:
        return "proposal"
