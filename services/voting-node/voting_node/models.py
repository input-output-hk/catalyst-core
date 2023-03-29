import yaml
from aiofile import async_open
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional

from .logs import getLogger

# gets voting node logger
logger = getLogger()


### Base types
@dataclass
class YamlType:
    content: Dict

    def as_yaml(self) -> str:
        return yaml.safe_dump(self.content)


@dataclass
class YamlFile:
    yaml_type: YamlType
    path: Path

    async def save(self):
        """YAML files are written asynchronously due to their possible size."""
        yaml_str: str = self.yaml_type.as_yaml()
        afp = await async_open(self.path, "w")
        await afp.write(yaml_str)
        await afp.close()


@dataclass
class ServiceSettings:
    # ports
    rest_port: int
    jrpc_port: int
    p2p_port: int
    # Jormungandr node storage directory
    storage: str
    # use JCli to make calls
    jcli_path_str: str
    # use Jormungandr to run the server
    jorm_path_str: str


@dataclass
class NodeConfig(YamlType):
    """Data for creating 'node_config.yaml'."""


### File types
@dataclass
class NodeConfigYaml(YamlFile):
    """Represents the contents and path to 'node_secret.yaml'."""

    path: Path
    yaml_type: NodeConfig


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
class LeaderHostInfo:
    """Peer information that leaders need for consensus."""

    hostname: str
    consensus_leader_id: str


@dataclass
class Block0:
    """Represents the path to 'block0.bin' and its hash."""

    bin: bytes
    hash: str

    async def save(self, path: Path):
        afp = await async_open(path, "wb")
        await afp.write(self.bin)
        await afp.close()


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
    # The name of the event. eg. "Fund9" or "SVE1"
    name: str
    # A detailed description of the purpose of the event. eg. the events "Goal"
    description: str

    # The Time (UTC) Registrations are taken from Cardano main net.
    # Registrations after this date are not valid for voting on the event.
    # NULL = Not yet defined or Not Applicable
    registration_snapshot_time: Optional[datetime]
    voting_power_threshold: Optional[int]
    max_voting_power_pct: Optional[int]

    # The Time (UTC) Registrations are taken from Cardano main net.
    # Registrations after this date are not valid for voting on the event.
    # NULL = Not yet defined or Not Applicable
    start_time: Optional[datetime]
    end_time: Optional[datetime]

    # The Time (UTC) Registrations taken from Cardano main net are considered stable.
    # This is not the Time of the Registration Snapshot,
    # This is the time after which the registration snapshot will be stable.
    # NULL = Not yet defined or Not Applicable
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
        """Gets the timestamp for the event start time.
        This method raises exception if the timestamp is None."""
        if self.start_time is None:
            raise Exception("event has no start time")
        return self.start_time

    def has_started(self) -> bool:
        """Returns True when current time is equal or greater
        to the event start time.
        This method raises exception if the timestamp is None."""
        start_time = self.get_start_time()
        now = datetime.utcnow()
        return now >= start_time

    def get_snapshot_start(self) -> datetime:
        """Gets the timestamp for when the event snapshot becomes stable.
        This method raises exception if the timestamp is None."""
        if self.snapshot_start is None:
            raise Exception("event has no snapshot start time")
        return self.snapshot_start

    def get_voting_start(self) -> datetime:
        """Gets the timestamp for when the event voting starts.
        This method raises exception if the timestamp is None."""
        if self.voting_start is None:
            raise Exception("event has no voting start time")
        return self.voting_start

    def has_voting_started(self) -> bool:
        """Returns True when current time is equal or greater
        to the event start time.
        This method raises exception if the timestamp is None."""
        voting_start = self.get_voting_start()
        now = datetime.utcnow()
        return now >= voting_start

    def get_block0(self) -> Block0:
        """Gets the block0 binary for the event.
        This method raises exception if the block0 is None."""
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
class Snapshot:
    row_id: int
    event: int
    as_at: datetime
    last_updated: datetime
    dbsync_snapshot_data: Optional[str]
    drep_data: Optional[str]
    catalyst_snapshot_data: Optional[str]
    final: bool


@dataclass
class FundsForToken:
    address: str
    value: int
    token_id: str


@dataclass
class VotingGroup:
    row_id: str
    # The ID of this group
    group_id: str
    # The event (row_id) this group belongs to
    event_id: int
    # The ID of the voting token used by this group
    token_id: Optional[str]


@dataclass
class Voter:
    row_id: str
    # Either the voting key
    voting_key: str
    # The ID of the snapshot this record belongs to
    snapshot_id: str
    # The voting group this voter belongs to
    voting_group: str
    # The voting power associated with this key
    voting_power: int


@dataclass
class VotePlan:
    row_id: str
    # The event (row_id) this plan belongs to
    event_id: int
    # The ID of the plan in the voting ledger/bulletin board.
    # A Binary value encoded as hex
    id: str
    # The kind of vote which can be cast on this vote plan
    category: str
    # The public encryption key used. ONLY if required by the voteplan category
    encryption_key: Optional[str]
    # The voting group row_id this plan belongs to
    # The identifier of voting power token used withing this plan
    group_id: Optional[int]


@dataclass
class VotePlanCertificate:
    vote_plan: VotePlan
    certificate: str


@dataclass
class VotingNode:
    # Path to the node's storage
    storage: Path = Path("node_storage")
    # Hostname, private/public keypair, and topology key.
    host_info: Optional[HostInfo] = None
    # Jormungandr `node_config.yaml` data
    config: Optional[NodeConfigYaml] = None
    # Jormungandr `node_secret.yaml` data
    secret: Optional[NodeSecretYaml] = None
    # Jormungandr `topology_key` data
    topology_key: Optional[NodeTopologyKey] = None
    # Jormungandr peer leaders
    leaders: Optional[List[LeaderHostInfo]] = None
    # Voting Event
    event: Optional[Event] = None
    # Block0 for the event
    block0: Optional[Block0] = None

    def reset(self):
        """Resets the current instance by re-initializing the object."""
        self.__init__()

    def get_event(self) -> Event:
        """Returns the voting event ID, raises exception if it is None."""
        if self.event is None:
            raise Exception("no voting event was found")
        return self.event

    def get_leaders(self) -> List[LeaderHostInfo]:
        """Returns the list of known leaders,
        raises exception if it is None or empty."""
        if self.leaders is None:
            raise Exception("no leaders were found")
        return self.leaders

    def get_event_id(self) -> int:
        """Returns the voting event ID, raises exception if it is None."""
        event = self.get_event()
        return event.row_id

    def get_start_time(self) -> datetime:
        """Gets the timestamp for the event start time.
        This method raises exception if the event or the timestamp are None."""
        event = self.get_event()
        return event.get_start_time()

    def get_snapshot_start(self) -> datetime:
        """Gets the timestamp for when the event snapshot becomes stable.
        This method raises exception if the event or the timestamp are None."""
        event = self.get_event()
        return event.get_snapshot_start()

    def get_block0(self) -> Block0:
        """Returns Block0 for this voting event, raises exception if it is None."""
        event = self.get_event()
        return event.get_block0()

    def get_voting_start(self) -> datetime:
        """Gets the timestamp for when the event snapshot becomes stable.
        This method raises exception if the event or the timestamp are None."""
        event = self.get_event()
        return event.get_voting_start()

    def has_started(self) -> bool:
        """Gets the timestamp for when the event snapshot becomes stable.
        This method raises exception if the event or the timestamp are None."""
        event = self.get_event()
        return event.has_started()

    def has_snapshot_started(self) -> bool:
        """Returns true when the current time is greater or equal to the
        event's snapshot_start timestamp."""
        snapshot_start = self.get_snapshot_start()
        return datetime.utcnow() > snapshot_start

    def has_voting_started(self) -> bool:
        """Gets the timestamp for when the event voting starts.
        This method raises exception if the event or the timestamp are None."""
        event = self.get_event()
        return event.has_voting_started()


@dataclass
class LeaderNode(VotingNode):
    ...


@dataclass
class Leader0Node(LeaderNode):
    genesis: Optional[Genesis] = None
    initial_fragments: Optional[List[FundsForToken | VotePlanCertificate]] = None
    proposals: Optional[List[Proposal]] = None


@dataclass
class Follower0Node(VotingNode):
    ...
