"""Database models for the vit-servicing-station database."""

from dataclasses import dataclass
from datetime import datetime
from typing import Any, ClassVar, List, Mapping, Optional, Set


@dataclass
class Model:
    """Base class for all models."""

    exclude_from_insert: ClassVar[Set[str]] = set()

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        raise NotImplementedError()


@dataclass
class Event(Model):
    """Represents a database objective."""

    name: str
    description: str
    registration_snapshot_time: Optional[datetime]
    voting_power_threshold: Optional[int]
    max_voting_power_pct: Optional[int]
    start_time: Optional[datetime]
    end_time: Optional[datetime]
    insight_sharing_start: Optional[datetime]
    proposal_submission_start: Optional[datetime]
    refine_proposals_start: Optional[datetime]
    finalize_proposals_start: Optional[datetime]
    proposal_assessment_start: Optional[datetime]
    assessment_qa_start: Optional[datetime]
    snapshot_start: Optional[datetime]
    voting_start: Optional[datetime]
    voting_end: Optional[datetime]
    tallying_end: Optional[datetime]
    extra: Optional[Mapping[str, Any]]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "event"


@dataclass
class Objective(Model):
    """Represents a database objective."""

    exclude_from_insert: ClassVar[Set[str]] = {"row_id"}

    row_id: int
    id: int
    event: int
    category: str
    title: str
    description: str
    deleted: bool
    rewards_currency: Optional[str]
    rewards_total: Optional[int]
    proposers_rewards: Optional[int]
    vote_options: Optional[int]
    extra: Optional[Mapping[str, Any]]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "objective"


@dataclass
class Proposal(Model):
    """Represents a database proposal."""

    id: int
    objective: int
    title: str
    summary: str
    deleted: bool
    category: str
    public_key: str
    funds: int
    url: str
    files_url: str
    impact_score: float

    extra: Optional[Mapping[str, str]]

    proposer_name: str
    proposer_contact: str
    proposer_url: str
    proposer_relevant_experience: str

    bb_proposal_id: Optional[bytes]
    bb_vote_options: Optional[List[str]]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "proposal"


@dataclass
class Goal(Model):
    """Represents a database goal."""

    event_id: int
    idx: int
    name: str

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "goal"


@dataclass
class VotingGroup(Model):
    """Represents a database voting_group."""

    group_id: str
    event_id: int
    token_id: Optional[str]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "voting_group"


@dataclass
class Voteplan(Model):
    """Represents a database voteplan."""

    event_id: int
    id: str
    category: str
    encryption_key: Optional[str]
    group_id: Optional[int]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "voteplan"


@dataclass
class ProposalVoteplan(Model):
    """Represents a database proposal_voteplan."""

    proposal_id: Optional[int]
    voteplan_id: Optional[int]
    bb_proposal_index: Optional[int]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "proposal_voteplan"


@dataclass
class Voter(Model):
    """Represents a database voter."""

    voting_key: str
    snapshot_id: int
    voting_group: str
    voting_power: int

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "voter"


@dataclass
class Contribution(Model):
    """Represents a database contribution."""

    stake_public_key: str
    snapshot_id: int
    voting_key: Optional[str]
    voting_weight: Optional[int]
    voting_key_idx: Optional[int]
    value: int
    voting_group: str
    reward_address: Optional[str]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "contribution"


@dataclass
class Snapshot(Model):
    """Represents a database snapshot."""

    exclude_from_insert: ClassVar[Set[str]] = {"row_id"}

    row_id: int
    event: int
    as_at: datetime
    as_at_slotno: int
    last_updated: datetime
    last_updated_slotno: int
    final: bool
    dbsync_snapshot_cmd: Optional[str]
    dbsync_snapshot_data: Optional[bytes]
    drep_data: Optional[bytes]
    catalyst_snapshot_cmd: Optional[str]
    catalyst_snapshot_data: Optional[bytes]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "snapshot"

@dataclass
class Config(Model):
    """Represents a database config."""

    row_id: int
    id: str
    id2: str
    id3: str
    value: Optional[Mapping[str, Any]]

    @staticmethod
    def table() -> str:
        """Return the name of the table that this model is stored in."""
        return "config"
