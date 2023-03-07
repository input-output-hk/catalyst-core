import yaml

from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional


@dataclass
class YamlType:
    content: Dict

    def as_yaml(self) -> str:
        return yaml.safe_dump(self.content)


@dataclass
class YamlFile:
    yaml_type: YamlType
    path: Path

    def save(self):
        yaml_str: str = self.yaml_type.as_yaml()
        self.path.open("w").write(yaml_str)


@dataclass
class NodeConfig(YamlType):
    """Data for creating 'node_config.yaml'."""


@dataclass
class NodeConfigYaml(YamlFile):
    """Represents the contents and path to 'node_secret.yaml'."""

    yaml_type: NodeConfig


@dataclass
class NodeSettings:
    rest_port: int
    jrpc_port: int
    p2p_port: int


@dataclass
class NodeInfo:
    hostname: str
    event: int
    seckey: str
    pubkey: str
    netkey: str


@dataclass
class NodeSecretYaml:
    """Represents the contents and path to 'node_secret.yaml'."""

    content: Dict
    path: Path

    def save(self):
        yaml_str: str = yaml.safe_dump(self.content)
        self.path.open("w").write(yaml_str)


@dataclass
class NodeTopologyKey:
    """Represents the contents and path to 'node_topology_key' file."""

    key: str
    path: Path

    def save(self):
        self.path.open("w").write(self.key)


@dataclass
class PeerNode:
    hostname: str
    ip_addr: str
    consensus_leader_id: str


@dataclass
class Block0:
    """Represents the path to 'block0.bin' and its hash."""

    bin_path: Path
    hash: str


@dataclass
class Genesis(YamlType):
    """Data for creating 'node_config.yaml'."""


@dataclass
class GenesisYaml(YamlFile):
    """Represents the contents and path to 'genesis.yaml'."""

    yaml_type: Genesis


@dataclass
class Event:
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

    block0: Optional[bytes]
    block0_hash: Optional[str]

    committee_size: int
    committee_threshold: int

    extra: Optional[Mapping[str, Any]]


@dataclass
class Proposal:
    """Represents a database proposal."""

    row_id: int
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
    node_info: NodeInfo
    node_config: NodeConfigYaml
    node_secret: NodeSecretYaml
    topology_key: NodeTopologyKey
    peer_info: List[PeerNode]
    voting_event: Event
    block0: Block0


@dataclass
class LeaderNode(VotingNode):
    ...


@dataclass
class Leader0Node(LeaderNode):
    genesis: Optional[Genesis]


@dataclass
class Follower0Node(VotingNode):
    ...
