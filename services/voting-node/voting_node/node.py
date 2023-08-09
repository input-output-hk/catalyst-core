"""Three types of nodes are defined.

* Leader0
* Leader
* Follower
"""
from datetime import datetime
from pathlib import Path

from loguru import logger
from pydantic import BaseModel

from .models import (
    Block0,
    Event,
    FundsForToken,
    Genesis,
    HostInfo,
    LeaderHostInfo,
    NodeConfigYaml,
    NodeSecretYaml,
    NodeTopologyKey,
    Proposal,
    VotePlanCertificate,
)
from .models.committee import Committee


class BaseNode(BaseModel):
    """Provides common functionality for all node types."""

    # Path to the node's storage
    # Path to the node's storage
    storage: Path = Path("node_storage")
    # Hostname, private/public keypair, and topology key.
    host_info: HostInfo | None = None
    # Jormungandr `node_config.yaml` data
    config: NodeConfigYaml | None = None
    # Jormungandr `node_secret.yaml` data
    secret: NodeSecretYaml | None = None
    # Jormungandr `topology_key` data
    topology_key: NodeTopologyKey | None = None
    # Jormungandr peer leaders
    leaders: list[LeaderHostInfo] | None = None
    # Voting Event
    event: Event | None = None
    # Block0 for the event
    block0: Block0 | None = None
    # Block0 for the event
    block0_path: Path | None = None

    def reset(self):
        """Reset the current instance by re-initializing the object."""
        self.__init__()

    def set_file_storage(self, path_str: str):
        """Initialize the path directory in case it doesn't exist."""
        storage = Path(path_str)
        storage.mkdir(parents=True, exist_ok=True)
        self.storage = storage
        logger.debug(f"Node Storage set to {path_str}")

    def get_event(self) -> Event:
        """Return the voting event ID, raises exception if it is None."""
        if self.event is None:
            raise Exception("no voting event was found")
        return self.event

    def get_leaders(self) -> list[LeaderHostInfo]:
        """Return the list of known leaders, raises exception if it is None or empty."""
        if self.leaders is None:
            raise Exception("no leaders were found")
        return self.leaders

    def get_event_id(self) -> int:
        """Return the voting event ID, raises exception if it is None."""
        event = self.get_event()
        return event.row_id

    def get_start_time(self) -> datetime:
        """Get the timestamp for the event start time.

        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.get_start_time()

    def get_registration_snapshot_time(self) -> datetime:
        """Get the timestamp for when the event registration has ended on the Cardano main net.

        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.get_registration_snapshot_time()

    def get_snapshot_start(self) -> datetime:
        """Get the timestamp for when the event snapshot becomes stable.

        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.get_snapshot_start()

    def get_block0(self) -> Block0:
        """Return Block0 for this voting event, raises exception if it is None."""
        event = self.get_event()
        return event.get_block0()

    def get_voting_start(self) -> datetime:
        """Get the timestamp for when voting starts.

        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.get_voting_start()

    def get_voting_end(self) -> datetime:
        """Get the timestamp for when voting ends.

        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.get_voting_end()

    def has_started(self) -> bool:
        """Get the timestamp for when the event snapshot becomes stable.

        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.has_started()

    def has_registration_ended(self) -> bool:
        """Return true when the current time is greater or equal to the event's registration_snapshot_time timestamp.

        This is the time when voter registration is closed.
        """
        registration_end = self.get_registration_snapshot_time()
        return datetime.utcnow() > registration_end

    def has_snapshot_started(self) -> bool:
        """Return true when the current time is greater or equal to the event's snapshot_start timestamp.

        This is the time when the snapshot is considered to be stable.
        """
        event = self.get_event()
        return event.has_snapshot_started()

    def has_voting_started(self) -> bool:
        """Get the timestamp for when the event voting starts.

        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.has_voting_started()

    def has_voting_ended(self) -> bool:
        """Get the timestamp for when the event voting ends, and the tallying beings.

        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.has_voting_ended()


class LeaderNode(BaseNode):
    """A leader node, excluding "leader0"."""

    ...


class Leader0Node(LeaderNode):
    """A leader0 node."""

    genesis: Genesis | None = None
    committee: Committee | None = None
    initial_fragments: list[FundsForToken | VotePlanCertificate] | None = None
    proposals: list[Proposal] | None = None

    def get_committee(self) -> Committee:
        """Return the Committee data, raises exception if it is None."""
        match self.committee:
            case Committee():
                return self.committee
            case _:
                raise Exception("node has no committee")


class FollowerNode(BaseNode):
    """A follower node."""
