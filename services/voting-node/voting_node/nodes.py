from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional


@dataclass
class NodeSettings:
    db_url: str
    jcli_path: str
    rest_port: int
    jrpc_port: int
    p2p_port: int


@dataclass
class NodeConfig:
    config: Dict
    path: Path


@dataclass
class NodeInfo:
    hostname: str
    seckey: str
    pubkey: str
    netkey: str


@dataclass
class NodeSecret:
    secret: Dict
    path: Path


@dataclass
class PeerNode:
    hostname: str
    ip_addr: str
    consensus_leader_id: str


@dataclass
class Block0:
    path: Path
    hash: str


@dataclass
class Genesis:
    settings: Dict
    path: Path


@dataclass
class Election:
    row_id: int
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


@dataclass
class Proposal:
    """
    Represents a database proposal.
    """

    id: int
    challenge: int
    title: str
    summary: str
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
    bb_vote_options: Optional[str]


@dataclass
class VotingNode:
    block0: Optional[Block0]
    election: Optional[Election]
    node_config: Optional[NodeConfig]
    node_secret: Optional[NodeSecret]
    peer_info: Optional[List[PeerNode]]


@dataclass
class LeaderNode(VotingNode):
    ...


@dataclass
class Leader0Node(LeaderNode):
    genesis: Optional[Genesis]


@dataclass
class Follower0Node(VotingNode):
    ...
