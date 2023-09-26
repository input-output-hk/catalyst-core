"""Tasks are run by the schedule runner.

Scheduled tasks are defined for Leader and Follower Nodes, with Leader0 being a special case,
as it is the only one responsible for initializing block0 for a voting event.
"""
import os
import secrets
from typing import Final, Mapping, NoReturn

from loguru import logger

from . import utils
from .db import EventDb
from .envvar import COMMITTEE_CRS
from .importer import SnapshotRunner
from .jcli import JCli
from .jormungandr import Jormungandr
from .models import (
    BftSigningKey,
    Block0,
    GenesisYaml,
    HostInfo,
    NodeConfigYaml,
    NodeSecretYaml,
    NodeTopologyKey,
    Objective,
    Proposal,
    ServiceSettings,
    Voter,
    VotingGroup,
    YamlType,
)
from .node import (
    BaseNode,
    FollowerNode,
    Leader0Node,
    LeaderNode,
)
from .storage import SecretDBStorage

RESET_DATA = False
KEEP_DATA = False
SCHEDULE_RESET_MSG = "schedule was reset"

LEADER_NODE_SCHEDULE: Final = [
    "node_fetch_event",
    "node_wait_for_start_time",
    "node_fetch_host_keys",
    "node_fetch_leaders",
    "node_set_secret",
    "node_set_topology_key",
    "node_set_config",
    "block0_fetch",
    "node_wait_for_voting",
    "event_voting_period",
    "node_wait_for_tally",
    "event_tally_period",
    "node_cleanup",
]
"""List of scheduled tasks for peer leader nodes."""
LEADER0_NODE_SCHEDULE: Final = [
    "node_fetch_event",
    "node_wait_for_start_time",
    "node_fetch_host_keys",
    "node_fetch_leaders",
    "node_set_secret",
    "node_set_topology_key",
    "node_set_config",
    "event_snapshot_period",
    "node_snapshot_data",
    "block0_tally_committee",
    "block0_voting_tokens",
    "block0_wallet_registrations",
    "block0_token_distributions",
    "block0_genesis_build",
    "block0_publish",
    "node_wait_for_voting",
    "event_voting_period",
    "node_wait_for_tally",
    "event_tally_period",
    "node_cleanup",
]
"""List of scheduled tasks for leader0 nodes."""
FOLLOWER_NODE_SCHEDULE: Final = [
    "node_fetch_event",
    "node_wait_for_start_time",
    "node_fetch_host_keys",
    "node_fetch_leaders",
    "node_set_secret",
    "node_set_topology_key",
    "node_set_config",
    "node_cleanup",
]
"""List of scheduled tasks for follower nodes."""


class ScheduleRunner:
    """Base class to run a sequential task list."""

    current_task: str | None = None
    tasks: list[str] = []

    def reset_data(self) -> None:
        """Reset data kept by the schedule runner."""
        self.current_task = None

    def reset_schedule(self, msg: str = SCHEDULE_RESET_MSG, reset_data: bool = RESET_DATA) -> NoReturn:
        """Reset the schedule by setting the current task to None.

        Raise an exception that can be handled by the calling service.
        This method never returns.
        """
        if reset_data:
            self.reset_data()
        raise Exception(f"{msg}")

    async def run(self) -> None:
        """Run through the scheduled tasks.

        Each task is executed and checked for exceptions. It is left to the task
        itself to check for exceptions or let them propagate, or to use the
        `rest_schedule` method to explicitly raise an exception with the options
        to add a custom error message, as well as the possibility to reset any data
        already stored in the schedule.

        This method is meant to be called from a conditional loop, so that the schedule
        will persist in finishing all the tasks from its list. But it can be called
        manually as well.
        """
        # checks if it should resume from a current task or go through all
        if self.current_task is None:
            tasks = self.tasks[:]
            logger.info("SCHEDULE START")
        else:
            task_idx = self.tasks.index(self.current_task)
            tasks = self.tasks[task_idx:]
            logger.info("SCHEDULE RESTART")

        for task in tasks:
            try:
                await self.run_task(task)
            except Exception as e:
                raise e
        logger.info("SCHEDULE END")

    async def run_task(self, task_name):
        """Run the async method with the given task_name."""
        logger.info(f">> TASK {task_name}")
        logger.debug(f"| {task_name} START")
        self.current_task = task_name
        task_exec = getattr(self, task_name)
        await task_exec()
        logger.debug(f"| {task_name} END")


class NodeTaskSchedule(ScheduleRunner):
    """A schedule of task names with corresponding async methods that are executed sequentially.

    If the current task raises an exception, running the task list again will resume from it.
    """

    # runtime settings for the service
    settings: ServiceSettings = ServiceSettings()
    # connection to DB
    db: EventDb = EventDb()
    # Voting Node data
    node: BaseNode = BaseNode()
    # List of tasks that the schedule should run. The name of each task must
    # match the name of method to execute.
    tasks: list[str] = []

    def __init__(self, settings: ServiceSettings = ServiceSettings()) -> None:
        """Set the schedule settings, bootstraps node storage, and initializes the node."""
        self.settings = settings
        self.db.db_url = settings.db_url
        self.node.set_file_storage(settings.storage)

    def reset_data(self) -> None:
        """Reset data for the node task schedule."""
        ScheduleRunner.reset_data(self)
        self.node.reset()

    def jcli(self) -> JCli:
        """Return the wrapper to the 'jcli' shell command."""
        return JCli(self.settings.jcli_path_str)

    def jorm(self) -> Jormungandr:
        """Return the wrapper to the 'jormungandr' shell command."""
        return Jormungandr(self.settings.jorm_path_str)

    async def node_fetch_event(self):
        """Fetch the event from the DB.

        This all starts by getting the event row that has the nearest
        `voting_start`, or an event that has stared and not ended.

        We query the DB to get the row, and store it in the node.
        """
        try:
            event = await self.db.fetch_upcoming_event()
            logger.debug("upcoming event retrieved from DB")
            self.node.event = event
        except Exception as e:
            self.reset_schedule(f"{e}")

    async def node_wait_for_start_time(self):
        """Wait for the event start time."""
        # check if the event has started, otherwise, resets the schedule
        # raises exception if the event has no start time defined
        if self.node.has_started():
            logger.debug("event has started")
        else:
            logger.debug("event has not started")
            self.reset_schedule(f"event will start on {self.node.get_start_time()}")

    async def node_fetch_host_keys(self):
        """Fetch or create voting node secret key information, by hostname, for the current event.

        Reset the schedule if no current event is defined.

        1. Fetch host keys from the `voting_node` table in EventDB.
            If found, save to the schedule node, else, generate and write to DB.
            Call `reset_schedule`.
        2. Fetch host keys from node storage.
            If found:
                Compare with host keys fetched from step 1.
                If ServiceSettings has the reloadable flag set to True:
                    Overwrite host info in storage with host info from DB. Use keys from node storage.
                else:
                    Log a warning about the difference, and use keys from node storage.
            else:
                Store host info from step 1.
        """
        # gets the event, raises exception if none is found.
        event = self.node.get_event()

        try:
            # gets host information from voting_node table for this event
            # raises exception if none is found.
            host_info: HostInfo = await self.db.fetch_leader_host_info(event.row_id)
            logger.debug(f"fetched node host info from DB: {host_info.hostname}")
            self.node.host_info = host_info
        except Exception as e:
            # fetching from DB failed
            msg = "unable to fetch node host info"
            logger.debug(f"{msg}: {e}")
            logger.warning(msg)
            # generate and insert host information into voting_node table
            hostname = utils.get_hostname()
            event_id = event.row_id
            logger.debug(f"generating '{hostname}' host info with jcli")
            # generate the keys
            seckey = await self.jcli().key_generate(secret_type="ed25519")
            pubkey = await self.jcli().key_to_public(seckey)
            netkey = await self.jcli().key_generate(secret_type="ed25519")
            host_info = HostInfo(hostname, event_id, seckey, pubkey, netkey)
            logger.debug(f"host info was generated: {host_info.hostname}")
            try:
                # we add the host info row
                # raises exception if unable.
                await self.db.insert_leader_host_info(host_info)
                # explicitly reset the schedule to ensure this task is run again.
                raise Exception("added node host info to DB")
            except Exception as e:
                self.reset_schedule(f"{e}")

    async def node_fetch_leaders(self):
        """Fetch from the DB host info for other leaders."""
        # gets the event, raises exception if none is found.
        event = self.node.get_event()
        try:
            # gets info for other leaders
            # raises exception if unable.
            leaders = await self.db.fetch_sorted_leaders_host_info(event.row_id)
            self.node.leaders = leaders
        except Exception as e:
            self.reset_schedule(f"{e}")

    async def node_set_secret(self):
        """Set the seckey from the host info and saves it to the node storage node_secret.yaml."""
        # get the node secret from HostInfo, if it's not found, reset
        match self.node.host_info:
            case HostInfo(seckey=sk):
                # the path to the node secret in storage
                node_secret_file = self.node.storage.joinpath("node_secret.yaml")
                if node_secret_file.exists():
                    # read the node secret file
                    node_secret_yaml = await NodeSecretYaml.read_file(node_secret_file)
                    # save in node
                    self.node.secret = node_secret_yaml
                    logger.debug(f"{node_secret_file} found and loaded")
                else:
                    logger.debug(f"{node_secret_file} does not exist")
                    # create and save the node_secret.yaml file from the host info
                    node_secret = BftSigningKey(sk)
                    # save in node
                    self.node.secret = NodeSecretYaml(node_secret, node_secret_file)
                    # write key to file
                    await self.node.secret.save()
                    logger.debug(f"{self.node.secret.path} saved")
            case _:
                self.reset_schedule("no node host info was found")

    async def node_set_topology_key(self):
        """Set the node network topology key."""
        match self.node.host_info:
            case HostInfo(netkey=netkey):
                # get private key for topology
                topokey_file = self.node.storage.joinpath("node_topology_key")
                # save in schedule
                self.node.topology_key = NodeTopologyKey(yaml_type=YamlType(content=netkey), path=topokey_file)
                # write key to file
                await self.node.topology_key.save()
            case _:
                self.reset_schedule("host info was not found for this node")

    async def node_set_config(self):
        """Set the node configuration."""
        # check that we have the info we need, otherwise, we reset
        if self.node.topology_key is None:
            self.reset_schedule("no node topology key was found")
        if self.node.leaders is None:
            self.reset_schedule("no leaders info was found")

        #  modify node config for all nodes
        host_name = utils.get_hostname()

        role_str = self.settings.role
        if role_str is None:
            role_str = host_name

        host_ip = utils.get_hostname_addr()
        role: utils.NodeRole = utils.parse_node_role(role_str)
        p2p_port = self.settings.p2p_port

        listen_rest = f"{host_ip}:{self.settings.rest_port}"
        listen_jrpc = f"{host_ip}:{self.settings.jrpc_port}"
        listen_p2p = f"/dns4/{host_name}/tcp/{p2p_port}"
        trusted_peers = []

        for peer in self.node.leaders:
            if role.name == "leader" and role.n == 0:
                # this node does not trust peers
                pass
            elif role.name == "leader":
                peer_role_str = peer.role
                if peer_role_str is None:
                    peer_role_str = peer.hostname

                peer_role = utils.parse_node_role(peer_role_str)
                # only append if peer digits are smaller than host digits
                # This is to say that a example node "leader000" will
                # append "leader00", but not "leader0000".
                if peer_role == "leader" and peer_role.n < role.n:
                    peer_addr = f"/dns4/{peer.hostname}/tcp/{p2p_port}"
                    trusted_peers.append({"address": peer_addr})
            elif role.name == "follower":
                # append all leaders
                peer_addr = f"/dns4/{peer.hostname}/tcp/{p2p_port}"
                trusted_peers.append({"address": peer_addr})

        # node config from default template
        config = utils.make_node_config(
            role,
            listen_rest,
            listen_jrpc,
            listen_p2p,
            trusted_peers,
            self.node.storage,
            self.node.topology_key.path,
        )

        # convert to yaml and save
        node_config_yaml = NodeConfigYaml(config, self.node.storage.joinpath("node_config.yaml"))
        await node_config_yaml.save()
        logger.debug("node config saved")
        self.node.config = node_config_yaml

    async def node_cleanup(self):
        """Execute cleanup chores to stop the voting node service.

        * ...
        """


class LeaderSchedule(NodeTaskSchedule):
    """The schedule of tasks for all leader nodes, except for leader0."""

    # Leader Node
    node: LeaderNode = LeaderNode()
    # Leader Node tasks
    tasks: list[str] = LEADER_NODE_SCHEDULE

    async def block0_fetch(self):
        """Get block0 information from the node event.

        Raises exception if the node has no leaders, or no event, or no event start time defined.
        """
        # initial checks for data that's needed
        if self.node.leaders is None:
            self.reset_schedule("peer leader info was not found")
        if self.node.event is None:
            self.reset_schedule("event was not found")
        if self.node.event.start_time is None:
            self.reset_schedule("event has no start time")

        # Path to block0.bin file
        block0_path = self.node.storage.joinpath("block0.bin")
        # Get the optional fields from the current event
        block0 = self.node.event.get_block0()
        # save Block0 to schedule
        self.node.block0 = block0
        # write the block bytes to file
        await self.node.block0.save(block0_path)
        self.node.block0_path = block0_path
        logger.debug(f"block0 found in voting event: {self.node.block0}")

    async def node_wait_for_voting(self):
        """Wait for the event voting time."""
        # get the voting start timestamp
        # raises an exception otherwise
        voting_start = self.node.get_voting_start()
        # check if now is after the snapshot start time
        if not self.node.has_voting_started():
            raise Exception(f"voting will start on {voting_start} UTC")

        logger.debug("voting has started")

    async def event_voting_period(self):
        """Execute jormungandr node for voting."""
        logger.debug(f"NODE: {self.node.secret}")
        if self.node.secret is None:
            self.reset_schedule("node has no node_secret.yaml")
        if self.node.config is None:
            self.reset_schedule("node has no node_config.yaml")
        if self.node.block0_path is None:
            self.reset_schedule("event has no block0.bin")
        await self.jorm().start_leader(self.node.secret.path, self.node.config.path, self.node.block0_path)

    async def node_wait_for_tally(self):
        """Wait for vote tally to begin."""
        # get the voting end timestamp
        # raises an exception otherwise
        voting_end = self.node.get_voting_end()
        # check if now is after the snapshot start time
        if not self.node.has_voting_ended():
            raise Exception(f"voting will start on {voting_end} UTC")

        logger.debug("voting has ended, tallying has begun")

    async def event_tally_period(self):
        """Execute the vote tally."""


class Leader0Schedule(LeaderSchedule):
    """The schedule of tasks for leader0 node."""

    # Leader0 Node
    node: Leader0Node = Leader0Node()
    # Leader0 Node tasks
    tasks: list[str] = LEADER0_NODE_SCHEDULE

    async def event_snapshot_period(self):
        """Timespan for continuous snapshot import from DBSync and IdeaScale.

        A snapshot is taken at intervals of `SNAPSHOT_INTERVAL_SECONDS` (default 1800 seconds).
        """
        event = self.node.get_event()
        registration_time = event.get_registration_snapshot_time()
        snapshot_start = event.get_snapshot_start()
        runner = SnapshotRunner(registration_snapshot_time=registration_time, snapshot_start=snapshot_start)
        logger.info("Execute snapshot runner for event in row {event_id}.", event_id=event.row_id)
        await runner.take_snapshots(event.row_id)

    async def node_snapshot_data(self):
        """Collect the snapshot data from EventDB."""
        # gets the event, raises exception if none is found.
        event = self.node.get_event()
        snapshot_start = event.get_snapshot_start()
        if not event.has_snapshot_started():
            raise Exception(f"snapshot will be stable on {snapshot_start} UTC")

        # check for this field before getting the data
        is_final = await self.db.check_if_snapshot_is_final(event.row_id)
        if not is_final:
            logger.warning("snapshot is not final")

        async def collect_voting_groups() -> list[VotingGroup]:
            groups = []
            try:
                groups = await self.db.fetch_voting_groups()
            except Exception:
                logger.exception("failed to collect voting groups")
            finally:
                return groups

        async def collect_voter_registrations() -> list[Voter]:
            """Collect voter registration information."""
            voters = []
            try:
                voters = await self.db.fetch_voters(event.row_id)
            except Exception:
                logger.exception("failed to collect voter registration")
            finally:
                return voters

        async def collect_contributions():
            contributions = []
            try:
                contributions = await self.db.fetch_contributions(event.row_id)
            except Exception:
                logger.exception("failed to collect voter contributions")
            finally:
                return contributions

        async def collect_objectives():
            objectives = []
            try:
                objectives = await self.db.fetch_objectives(event.row_id)
            except Exception:
                logger.exception("failed to collect objectives")
            finally:
                return objectives

        async def collect_proposals(objectives: list[Objective]) -> Mapping[int, list[Proposal]]:
            objective_proposals: Mapping[int, list[Proposal]] = {}
            for objective in objectives:
                objective_id = objective.row_id
                try:
                    proposals = await self.db.fetch_proposals(objective_id)
                    objective_proposals[objective_id] = proposals
                except Exception:
                    logger.exception("failed to collect proposals for objective {objective_row}", objective_row=objective_id)
            return objective_proposals

        # Collect registration information needed for block0
        await collect_voting_groups()
        await collect_voter_registrations()
        await collect_contributions()

        # Collect voteplan information needed for block0
        objectives = await collect_objectives()
        objective_proposals = await collect_proposals(objectives)

    async def block0_tally_committee(self):
        """Fetch or create tally committee data.

        1. Fetch the committee from the node secret storage
        2. If no committee is found, create it.
            When creating the committee member keys, a Common Reference String, or CRS, is used.
            The CRS is expected to be in the COMMITTEE_CRS environment variable, if it is not set,
            then a random 32-byte hex token is generated and used.
        """
        event = self.node.get_event()
        conn = await self.db.connect()
        secret_store = SecretDBStorage(conn=conn)
        # TODO: fetch tally committee data from secret storage
        try:
            committee = await secret_store.get_committee(event.row_id)
            logger.debug("fetched committee from storage")
            self.node.committee = committee
        except Exception as e:
            logger.warning(f"failed to fetch committee from storage: {e}")
            # create the commitee for this event
            logger.debug("creating committee wallet info")
            committee_wallet = await utils.create_wallet_keyset(self.jcli())
            logger.debug("creating committee")
            # get CRS from the environment, or create a 32-byte hex token
            crs = os.environ.get(COMMITTEE_CRS)
            if crs is None:
                crs = secrets.token_hex(32)
            committee = await utils.create_committee(
                self.jcli(),
                event.row_id,
                event.committee_size,
                event.committee_threshold,
                crs,
            )
            logger.debug(f"created committee: {committee.as_yaml()}")
            self.node.committee = committee
            await secret_store.save_committee(event_id=event.row_id, committee=committee)
            logger.debug("saved committee to storage")
        finally:
            await conn.close()

    async def block0_voting_tokens(self):
        """Build voting tokens for each voting group in the current event."""
        logger.error("block0_voting_tokens is not implemented!")

    async def block0_wallet_registrations(self):
        """Build initial fund fragments for registered wallets."""
        logger.error("block0_wallet_registrations is not implemented!")

    async def block0_token_distributions(self):
        """Build initial token distribution fragments for registered contributions."""
        logger.error("block0_token_distributions is not implemented!")

    async def block0_genesis_build(self):
        """Build genesis block.

        Create and add it to the DB when the needed information is found.
        Reset the schedule otherwise.
        """
        # get or raise exception for data that's needed
        # these exceptions are not handled so as to keep
        # data already saved in the schedule.
        event = self.node.get_event()
        if not self.node.has_started():
            self.reset_schedule("event has not started")
        leaders = self.node.get_leaders()

        # look for block0 in the event from the DB
        # if found, decode the genesis.yaml, and
        # store the files.
        try:
            # Path where block0.bin file is kept
            block0_path = self.node.storage.joinpath("block0.bin")
            # Get the optional fields from the current event
            # raises an exception otherwise
            block0 = event.get_block0()
            # write the block bytes to file
            await block0.save(block0_path)
            # save Block0 to node
            self.node.block0 = block0
            self.node.block0_path = block0_path
            logger.debug(f"block0 found in voting event: {self.node.block0.hash}")

            # decode genesis and store it after getting block0
            genesis_path = self.node.storage.joinpath("genesis.yaml")
            await self.jcli().genesis_decode(block0_path, genesis_path)
            logger.debug(f"decoded and stored file: {genesis_path}")
        except Exception as e:
            logger.debug(f"block0 was not found in event: {e}")
            logger.debug("attempting to create block0")
            # checks for data that's needed
            if event.start_time is None:
                self.reset_schedule("event has no start time")

            # generate genesis file to make block0
            logger.debug("generating genesis content")
            committee = self.node.get_committee()
            genesis = utils.make_genesis_content(event, leaders, [committee.committee_id])
            logger.debug("generated genesis content")
            # convert to yaml and save
            genesis_path = self.node.storage.joinpath("genesis.yaml")
            genesis_yaml = GenesisYaml(genesis, genesis_path)
            await genesis_yaml.save()
            logger.debug(f"{genesis_yaml}")
            self.genesis_yaml = genesis_yaml

            block0_path = self.node.storage.joinpath("block0.bin")
            await self.jcli().genesis_encode(block0_path, genesis_path)

            block0_hash = await self.jcli().genesis_hash(block0_path)
            block0_bytes = block0_path.read_bytes()
            block0 = Block0(block0_bytes, block0_hash)
            self.node.block0 = block0
            # write the block bytes to file
            await self.node.block0.save(block0_path)
            self.node.block0_path = block0_path
            logger.debug(f"block0 created and saved: {self.node.block0.hash}")

    async def block0_publish(self):
        """Publish block0 to the current event in EventDB."""
        event = self.node.get_event()
        match self.node.block0:
            case Block0(bin=block0_bytes, hash=block0_hash):
                # push block0 to event table
                await self.db.insert_block0_info(event.row_id, block0_bytes, block0_hash)
                # if all is good, we reset the schedule
                logger.debug("inserted block0 info")
            case None:
                logger.error("expected block0 info")
                self.reset_schedule("node has no block0")


class FollowerSchedule(NodeTaskSchedule):
    """The schedule of tasks for follower nodes."""

    # Follower Node
    node: FollowerNode = FollowerNode()
    # Follower Node tasks
    tasks: list[str] = FOLLOWER_NODE_SCHEDULE
