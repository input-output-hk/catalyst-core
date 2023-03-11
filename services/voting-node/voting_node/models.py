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
class HostInfo:
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
    """Peer information that leaders need for consensus."""

    hostname: str
    consensus_leader_id: str


@dataclass
class Block0:
    """Represents the path to 'block0.bin' and its hash."""

    bin: bytes
    hash: str


@dataclass
class Block0File:
    block0: Block0
    path: Path

    def save(self):
        self.path.write_bytes(self.block0.bin)


@dataclass
class Genesis(YamlType):
    """Data for creating 'genesis.yaml'."""


@dataclass
class GenesisYaml(YamlFile):
    """Represents the contents and path to 'genesis.yaml'."""

    yaml_type: Genesis


@dataclass
class Event:
    """Represents DB row for the current event."""

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

    def get_start_time(self) -> datetime:
        if self.start_time is None:
            raise Exception("event has no start time")
        return self.start_time

    def get_block0(self) -> Block0:
        if self.block0 is None or self.block0_hash is None:
            raise Exception("event has no block0")
        block0: bytes = self.block0
        block0_hash: str = self.block0_hash
        return Block0(block0, block0_hash)


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
    host_info: Optional[HostInfo] = None
    node_config: Optional[NodeConfigYaml] = None
    node_secret: Optional[NodeSecretYaml] = None
    topology_key: Optional[NodeTopologyKey] = None
    leaders: Optional[List[PeerNode]] = None
    event: Optional[Event] = None
    block0: Optional[Block0File] = None

    def reset(self):
        self.__init__()


@dataclass
class LeaderNode(VotingNode):
    ...


@dataclass
class Leader0Node(LeaderNode):
    genesis: Optional[Genesis] = None
    proposals: Optional[List[Proposal]] = None


@dataclass
class Follower0Node(VotingNode):
    ...
