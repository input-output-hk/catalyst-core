"""Mostly data models with convenience methods."""
from collections.abc import Mapping
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Optional, Self

import yaml
from aiofile import async_open


### Base types
@dataclass
class YamlType:
    """A convenience type for YAML objects."""

    content: dict | str

    def as_yaml(self) -> str:
        """Return the content as YAML."""
        match self.content:
            case str(_):
                return self.content
            case dict(_):
                return yaml.safe_dump(self.content)


@dataclass
class YamlFile:
    """A convenience type for YAML objects that are saved as files."""

    yaml_type: YamlType
    path: Path

    async def save(self):
        """Save YAML content to the file path.

        YAML files are written asynchronously due to their possible size.
        """
        yaml_str: str = self.yaml_type.as_yaml()
        afp = await async_open(self.path, "w")
        await afp.write(yaml_str)
        await afp.close()


@dataclass
class ServiceSettings:
    """Settings for the node service."""

    # ports
    rest_port: int = 10080
    """REST listening port for jormungandr."""
    jrpc_port: int = 10085
    """REST listening port for jormungandr."""
    p2p_port: int = 10090
    """P2P listening port for jormungandr."""
    storage: str = "./node_storage"
    """Jormungandr node storage directory."""
    # use JCli to make calls
    jcli_path_str: str = "jcli"
    """Path to jcli executable."""
    # use Jormungandr to run the server
    jorm_path_str: str = "jormungandr"
    """Path to jormungandr executable."""
    # URL to Event DB
    db_url: str = "postgres://catalyst-event-dev:CHANGE_ME@postgres/CatalystEventDev"
    """URL to Event DB."""
    # Should the service reload if the current event
    # has changed.
    reloadable: bool = False
    """Enable resetting and reloading the node service during runtime."""
    role: Optional[str] = None
    """Specify which role the voting node should assume (e.g. leader0).
    If not specified, the role will be defined by the node's hostname."""


@dataclass
class NodeConfig(YamlType):
    """Data for creating 'node_config.yaml'."""


### File types
@dataclass
class NodeConfigYaml(YamlFile):
    """Represents the contents and path to 'node_secret.yaml'."""

    yaml_type: NodeConfig


@dataclass
class HostInfo:
    """Node host information for a given event.

    Holds the hostname, the node's keypair, and topology key.
    """

    hostname: str
    event: int
    seckey: str
    pubkey: str
    netkey: str


@dataclass
class BftSigningKey(YamlType):
    """BFT Signing Key is the node secret key."""

    content: str

    def to_node_secret_dict(self) -> dict:
        """Return the signing key as the node secret dictionary."""
        return {"bft": {"signing_key": self.content}}

    def as_yaml(self) -> str:
        """Return the content as YAML."""
        return yaml.safe_dump(self.to_node_secret_dict())


@dataclass
class NodeSecretYaml(YamlFile):
    """Represents the contents and path to 'node_secret.yaml'."""

    yaml_type: BftSigningKey

    @classmethod
    async def read_file(cls, file: Path) -> Self:
        """Read and return the BFT signing key from the file path."""
        afp = await async_open(file, "r")
        yaml_str = await afp.read()
        await afp.close()
        yaml_dict = yaml.safe_load(yaml_str)
        match yaml_dict:
            case {"bft": {"signing_key": seckey}}:
                return cls(yaml_type=BftSigningKey(content=seckey), path=file)
            case _:
                raise Exception(f"invalid node secret in {file}")


@dataclass
class NodeTopologyKey(YamlFile):
    """Represents the contents and path to node_topology_key file."""


@dataclass
class LeaderHostInfo:
    """Peer information that leaders need for consensus."""

    hostname: str
    consensus_leader_id: str
    role: Optional[str]


@dataclass
class Block0:
    """Represents the path to 'block0.bin' and its hash."""

    bin: bytes
    hash: str

    async def save(self, path: Path):
        """Save the block0 binary file to the specified path."""
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

    # The primary key in the DB.
    row_id: int
    # The name of the event. eg. "Fund9" or "SVE1".
    name: str
    # A detailed description of the purpose of the event. eg. the events "Goal".
    description: str

    committee_size: int
    committee_threshold: int

    # The Time (UTC) Registrations are taken from Cardano main net.
    # Registrations after this date are not valid for voting on the event.
    # NULL = Not yet defined or Not Applicable
    registration_snapshot_time: datetime | None = None
    voting_power_threshold: int | None = None
    max_voting_power_pct: int | None = None
    """The Minimum number of Lovelace staked at the time of snapshot, to be eligible to vote.

    `None` means that it is not yet defined.
    """

    review_rewards: int | None = None
    """The total reward pool to pay for community reviewers for their valid reviews of the proposals assigned to this event."""

    # The Time (UTC) Registrations are taken from Cardano main net.
    # Registrations after this date are not valid for voting on the event.
    # NULL = Not yet defined or Not Applicable
    start_time: datetime | None = None
    end_time: datetime | None = None

    # The Time (UTC) Registrations taken from Cardano main net are considered stable.
    # This is not the Time of the Registration Snapshot,
    # This is the time after which the registration snapshot will be stable.
    # NULL = Not yet defined or Not Applicable
    insight_sharing_start: datetime | None = None
    proposal_submission_start: datetime | None = None
    refine_proposals_start: datetime | None = None
    finalize_proposals_start: datetime | None = None
    proposal_assessment_start: datetime | None = None
    assessment_qa_start: datetime | None = None
    snapshot_start: datetime | None = None
    voting_start: datetime | None = None
    voting_end: datetime | None = None
    tallying_end: datetime | None = None

    block0: bytes | None = None
    block0_hash: str | None = None

    extra: Mapping[str, Any] | None = None
    cast_to: Mapping[str, Any] | None = None

    def get_start_time(self) -> datetime:
        """Get the timestamp for the event start time.

        This method raises exception if the timestamp is None.
        """
        if self.start_time is None:
            raise Exception("event has no start time")
        return self.start_time

    def has_started(self) -> bool:
        """Return True when current time is equal or greater to the event start time.

        This method raises exception if the timestamp is None.
        """
        start_time = self.get_start_time()
        now = datetime.utcnow()
        return now >= start_time

    def get_registration_snapshot_time(self) -> datetime:
        """Get the timestamp for when the event registration has ended on the Cardano main net.

        This method raises exception if the timestamp is None.
        """
        if self.registration_snapshot_time is None:
            raise Exception("event has no registration snapshot time")
        return self.registration_snapshot_time

    def get_snapshot_start(self) -> datetime:
        """Get the timestamp for when the event snapshot becomes stable.

        This method raises exception if the timestamp is None.
        """
        if self.snapshot_start is None:
            raise Exception("event has no snapshot start time")
        return self.snapshot_start

    def has_snapshot_started(self) -> bool:
        """Return True when current time is equal or greater to the voting start time.

        This method raises exception if the timestamp is None.
        """
        snapshot_start = self.get_snapshot_start()
        return datetime.utcnow() >= snapshot_start

    def get_voting_start(self) -> datetime:
        """Get the timestamp for when the event voting starts.

        This method raises exception if the timestamp is None.
        """
        if self.voting_start is None:
            raise Exception("event has no voting start time")
        return self.voting_start

    def get_voting_end(self) -> datetime:
        """Get the timestamp for when the event voting ends.

        This method raises exception if the timestamp is None.
        """
        if self.voting_end is None:
            raise Exception("event has no voting end time")
        return self.voting_end

    def has_voting_started(self) -> bool:
        """Return True when current time is equal or greater to the voting start time.

        This method raises exception if the timestamp is None.
        """
        voting_start = self.get_voting_start()
        now = datetime.utcnow()
        return now >= voting_start

    def has_voting_ended(self) -> bool:
        """Return True when current time is equal or greater to the voting end time.

        This method raises exception if the timestamp is None.
        """
        voting_end = self.get_voting_end()
        now = datetime.utcnow()
        return now >= voting_end

    def get_block0(self) -> Block0:
        """Get the block0 binary for the event.

        This method raises exception if the block0 is None.
        """
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
    objective: int
    title: str
    summary: str
    category: str
    public_key: str
    funds: int
    url: str
    files_url: str
    impact_score: float
    proposer_name: str
    proposer_contact: str
    proposer_url: str
    proposer_relevant_experience: str
    extra: Mapping[str, str] | None = None
    bb_proposal_id: bytes | None = None
    bb_vote_options: list[str] | None = None


@dataclass
class Snapshot:
    """The snapshot row for the current event."""

    row_id: int
    event: int
    as_at: datetime
    last_updated: datetime
    dbsync_snapshot_data: bytes | None
    drep_data: str | None
    catalyst_snapshot_data: str | None
    final: bool


@dataclass
class FundsForToken:
    """Token distribution for initial fragments."""

    address: str
    value: int
    token_id: str


@dataclass
class VotingGroup:
    name: str
    """The ID of this voting group."""


@dataclass
class Voter:
    """A registered voter for this event."""

    row_id: int
    # Either the voting key
    voting_key: str
    # The ID of the snapshot this record belongs to
    snapshot_id: int
    # The voting group this voter belongs to
    voting_group: str
    # The voting power associated with this key
    voting_power: int


@dataclass
class Objective:
    row_id: int
    id: int
    event: int
    category: str
    title: str
    description: str
    deleted: bool
    rewards_currency: str | None = None
    rewards_total: int | None = None
    rewards_total_lovelace: int | None = None
    proposers_rewards: int | None = None
    vote_options: int | None = None

    extra: Mapping[str, Any] | None = None


@dataclass
class Contribution:
    """Individual contributions from the stake public key to the voting key."""

    row_id: int
    # Stake Public key for the voter.
    stake_public_key: str
    # The ID of the snapshot this record belongs to
    snapshot_id: int
    # The group that this contribution goes to.
    voting_group: str
    # The amount of ADA contributed to this voting key from the stake address.
    value: int

    # The voting key. If None, it is the raw staked ADA.
    voting_key: str | None = None
    # The weight that this key gets from the total.
    voting_weight: int | None = None
    # The index from 0 of the keys in the delegation array.
    voting_key_idx: int | None = None
    # Currently unused.
    reward_address: str | None = None


@dataclass
class VotePlan:
    """A vote plan for this event."""

    row_id: int
    # The event (row_id) this plan belongs to
    event_id: int
    # The ID of the plan in the voting ledger/bulletin board.
    # A Binary value encoded as hex
    id: str
    # The kind of vote which can be cast on this vote plan
    category: str
    # The public encryption key used. ONLY if required by the voteplan category
    encryption_key: str | None
    # The voting group row_id this plan belongs to
    # The identifier of voting power token used withing this plan
    group_id: int | None


@dataclass
class VotePlanCertificate:
    """A vote plan certificate for this event."""

    vote_plan: VotePlan
    certificate: str
