"""Three types of nodes are defined:

* Leader0

* Leader

* Follower

"""
from datetime import datetime
from pathlib import Path

from pydantic import BaseModel

from .logs import getLogger
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

# gets voting node logger
logger = getLogger()


class BaseNode(BaseModel):
    """Provides base functionality for a node."""

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

    def reset(self):
        """Resets the current instance by re-initializing the object."""
        self.__init__()

    def set_file_storage(self, path_str: str):
        """Initializes the path directory in case it doesn't exist."""
        storage = Path(path_str)
        storage.mkdir(parents=True, exist_ok=True)
        self.storage = storage
        logger.debug(f"Node Storage set to {path_str}")

    def get_event(self) -> Event:
        """Returns the voting event ID, raises exception if it is None."""
        if self.event is None:
            raise Exception("no voting event was found")
        return self.event

    def get_leaders(self) -> list[LeaderHostInfo]:
        """Returns the list of known leaders,
        raises exception if it is None or empty.
        """
        if self.leaders is None:
            raise Exception("no leaders were found")
        return self.leaders

    def get_event_id(self) -> int:
        """Returns the voting event ID, raises exception if it is None."""
        event = self.get_event()
        return event.row_id

    def get_start_time(self) -> datetime:
        """Gets the timestamp for the event start time.
        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.get_start_time()

    def get_snapshot_start(self) -> datetime:
        """Gets the timestamp for when the event snapshot becomes stable.
        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.get_snapshot_start()

    def get_block0(self) -> Block0:
        """Returns Block0 for this voting event, raises exception if it is None."""
        event = self.get_event()
        return event.get_block0()

    def get_voting_start(self) -> datetime:
        """Gets the timestamp for when the event snapshot becomes stable.
        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.get_voting_start()

    def has_started(self) -> bool:
        """Gets the timestamp for when the event snapshot becomes stable.
        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.has_started()

    def has_snapshot_started(self) -> bool:
        """Returns true when the current time is greater or equal to the
        event's snapshot_start timestamp.
        """
        snapshot_start = self.get_snapshot_start()
        return datetime.utcnow() > snapshot_start

    def has_voting_started(self) -> bool:
        """Gets the timestamp for when the event voting starts.
        This method raises exception if the event or the timestamp are None.
        """
        event = self.get_event()
        return event.has_voting_started()


class LeaderNode(BaseNode):
    ...


class Leader0Node(LeaderNode):
    genesis: Genesis | None = None
    initial_fragments: list[FundsForToken | VotePlanCertificate] | None = None
    proposals: list[Proposal] | None = None


class FollowerNode(BaseNode):
    ...
