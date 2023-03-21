from datetime import datetime
from pathlib import Path
from typing import Final, NoReturn, Optional

from . import utils
from .db import EventDb
from .jcli import JCli
from .jormungandr import Jormungandr
from .logs import getLogger
from .models import (
    Block0,
    GenesisYaml,
    Leader0Node,
    LeaderNode,
    NodeConfigYaml,
    HostInfo,
    NodeSecretYaml,
    NodeTopologyKey,
    ServiceSettings,
    VotingNode,
)

# gets voting node logger
logger = getLogger()

RESET_DATA = True
KEEP_DATA = False
SCHEDULE_RESET_MSG = "schedule was reset"

LEADER_NODE_SCHEDULE: Final = [
    "connect_db",
    "fetch_upcoming_event",
    "wait_for_start_time",
    "setup_host_info",
    "fetch_leaders",
    "set_node_secret",
    "set_node_topology_key",
    "set_node_config",
    "get_block0",
    "wait_for_voting",
    "voting",
    "wait_for_tally",
    "tally",
    "cleanup",
]
LEADER0_NODE_SCHEDULE: Final = [
    "connect_db",
    "fetch_upcoming_event",
    "wait_for_start_time",
    "setup_host_info",
    "fetch_leaders",
    "set_node_secret",
    "set_node_topology_key",
    "set_node_config",
    "wait_for_snapshot",
    "collect_snapshot_data",
    "setup_block0",
    "publish_block0",
    "wait_for_voting",
    "voting",
    "wait_for_tally",
    "tally",
    "cleanup",
]
FOLLOWER_NODE_SCHEDULE: Final = [
    "connect_db",
    "fetch_upcoming_event",
    "wait_for_start_time",
    "setup_host_info",
    "fetch_leaders",
    "set_node_secret",
    "set_node_topology_key",
    "set_node_config",
    "cleanup",
]


class ScheduleRunner(object):
    current_task: Optional[str] = None
    tasks: list[str] = []

    def reset_data(self) -> None:
        """Resets data kept by the schedule runner."""
        self.current_task = None

    def reset_schedule(
        self, msg: str = SCHEDULE_RESET_MSG, reset_data: bool = RESET_DATA
    ) -> NoReturn:
        """Reset the schedule by setting the current task to None, and raising
        an exception that can be handled by the calling service.

        This method never returns."""
        if reset_data:
            self.reset_data()
        raise Exception(f"|->{msg}")

    async def run(self) -> None:
        """Runs through the scheduled tasks.

        Each task is executed and checked for exceptions. It is left to the task
        itself to check for exceptions or let them propagate, or to use the
        `rest_schedule` method to explicitly raise an exception with the options
        to add a custom error message, as well as the possibility to reset any data
        already stored in the schedule.

        This method is meant to be called from a conditional loop, so that the schedule
        will persist in finishing all the tasks from its list. But it can be called
        manually as well."""
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
                raise Exception(f"'{task}': {e}") from e
        logger.info("SCHEDULE END")

    async def run_task(self, task_name):
        """Runs the async method with the given task_name."""
        logger.info(f"{task_name}")
        logger.debug(f"|'{task_name}' start")
        self.current_task = task_name
        task_exec = getattr(self, task_name)
        await task_exec()
        logger.debug(f"|'{task_name}' end")


class NodeTaskSchedule(ScheduleRunner):
    """A schedule of task names with corresponding async methods that are executed
    sequentially.

    If the current task raises an exception, running the task list
    again will resume from it."""

    # runtime settings for the service
    settings: ServiceSettings
    # connection to DB
    db: EventDb
    # Voting Node data
    node: VotingNode = VotingNode()
    # List of tasks that the schedule should run. The name of each task must
    # match the name of method to execute.
    tasks: list[str] = []

    def __init__(self, db_url: str, settings: ServiceSettings) -> None:
        self.settings = settings
        self.db = EventDb(db_url)
        self.node.storage = Path(settings.storage)
        # initialize the dir in case it doesn't exist
        self.node.storage.mkdir(parents=True, exist_ok=True)

    # resets data for the node task schedule
    def reset_data(self) -> None:
        ScheduleRunner.reset_data(self)
        self.node.reset()

    async def connect_db(self):
        # connect to the DB
        await self.db.connect()

    def jcli(self) -> JCli:
        return JCli(self.settings.jcli_path_str)

    def jorm(self) -> Jormungandr:
        return Jormungandr(self.settings.jorm_path_str)

    async def setup_host_info(self):
        try:
            # gets the event, raises exception if none is found.
            event = self.node.get_event()
        except Exception as e:
            self.reset_schedule(f"{e}")

        try:
            # gets host information from voting_node table for this event
            # raises exception if none is found.
            host_info: HostInfo = await self.db.fetch_leader_host_info(event.row_id)
            logger.debug("fetched node host info from DB")
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
            seckey = await self.jcli().privkey(secret_type="ed25519")
            pubkey = await self.jcli().pubkey(seckey)
            netkey = await self.jcli().privkey(secret_type="ed25519")
            host_info = HostInfo(hostname, event_id, seckey, pubkey, netkey)
            logger.debug("host info was generated")
            try:
                # we add the host info row
                # raises exception if unable.
                await self.db.insert_leader_host_info(host_info)
                # explicitly reset the schedule to ensure this task is run again.
                self.reset_schedule("added node host info to DB")
            except Exception as e:
                self.reset_schedule(f"{e}")

    async def fetch_leaders(self):
        try:
            # gets info for other leaders
            # raises exception if unable.
            leaders = await self.db.fetch_leaders_host_info()
            self.node.leaders = leaders
        except Exception as e:
            self.reset_schedule(f"{e}")

    async def set_node_secret(self):
        # node secret
        match self.node.host_info:
            case HostInfo(seckey=seckey):
                node_secret_file = self.node.storage.joinpath("node_secret.yaml")
                node_secret = {"bft": {"signing_key": seckey}}
                # save in schedule
                self.node_secret = NodeSecretYaml(node_secret, node_secret_file)
                # write key to file
                self.node_secret.save()
                logger.debug(f"{self.node_secret.path} saved")
            case _:
                self.reset_schedule("no node host info was found")

    async def set_node_topology_key(self):
        # node network topology key
        match self.node.host_info:
            case HostInfo(netkey=netkey):
                # get private key for topology
                topokey_file = self.node.storage.joinpath("node_topology_key")
                # save in schedule
                self.node.topology_key = NodeTopologyKey(netkey, topokey_file)
                # write key to file
                self.node.topology_key.save()
            case _:
                self.reset_schedule("host info was not found for this node")

    async def fetch_upcoming_event(self):
        # This all starts by getting the event row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        try:
            event = await self.db.fetch_upcoming_event()
            logger.debug("current event retrieved from DB")
            self.node.event = event
        except Exception as e:
            self.reset_schedule(f"{e}")

    async def wait_for_start_time(self):
        # get the event start timestamp
        # raises an exception otherwise
        start_time = self.node.get_start_time()
        # check if now is after the event start time
        if datetime.utcnow() >= start_time:
            logger.debug("event has started")
        else:
            logger.debug("event has not started")
            self.reset_schedule(f"event will start on {start_time}")

    async def set_node_config(self):
        # check that we have the info we need, otherwise, we reset
        if self.node.topology_key is None:
            self.reset_schedule("no node topology key was found")
        if self.node.leaders is None:
            self.reset_schedule("no leaders info was found")

        #  modify node config for all nodes
        host_name = utils.get_hostname()
        host_ip = utils.get_hostname_addr()
        role_n_number = utils.get_leadership_role_n_number_by_hostname(host_name)
        logger.debug(f"{role_n_number} ip: {host_ip}")
        p2p_port = self.settings.p2p_port

        listen_rest = f"{host_ip}:{self.settings.rest_port}"
        listen_jrpc = f"{host_ip}:{self.settings.jrpc_port}"
        listen_p2p = f"/dns4/{host_name}/tcp/{p2p_port}"
        trusted_peers = []

        for peer in self.node.leaders:
            match role_n_number:
                case ("leader", 0):
                    pass
                case ("leader", host_number):
                    match utils.get_leadership_role_n_number_by_hostname(peer.hostname):
                        case ("leader", peer_number):
                            if peer_number < host_number:
                                peer_addr = f"/dns4/{peer.hostname}/tcp/{p2p_port}"
                                trusted_peers.append({"address": peer_addr})
                        case _:
                            pass
                case ("follower", host_number):
                    peer_addr = f"/dns4/{peer.hostname}/tcp/{p2p_port}"
                    trusted_peers.append({"address": peer_addr})

        # node config from default template
        config = utils.make_node_config(
            role_n_number,
            listen_rest,
            listen_jrpc,
            listen_p2p,
            trusted_peers,
            self.node.storage,
            self.node.topology_key.path,
        )

        # convert to yaml and save
        node_config_yaml = NodeConfigYaml(
            config, self.node.storage.joinpath("node_config.yaml")
        )
        await node_config_yaml.save()
        logger.debug(f"{node_config_yaml}")

    async def cleanup(self):
        # close the DB connection
        await self.db.close()


class LeaderSchedule(NodeTaskSchedule):
    # Voting Node data
    node: LeaderNode = LeaderNode()
    # Tasks for `LeaderNode`
    tasks: list[str] = LEADER_NODE_SCHEDULE

    async def get_block0(self):
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
        logger.debug(f"block0 found in voting event: {self.node.block0}")

    async def wait_for_voting(self):
        """Waits for the event voting time."""
        # get the snapshot start timestamp
        # raises an exception otherwise
        voting_start = self.node.get_voting_start()
        # check if now is after the snapshot start time
        if not self.node.has_voting_started():
            raise Exception(f"voting will start on {voting_start} UTC")

        logger.debug("voting has started")

    async def wait_for_tally(self):
        pass

    async def voting(self):
        pass

    async def tally(self):
        pass


class Leader0Schedule(LeaderSchedule):
    # Voting Node data
    node: Leader0Node = Leader0Node()
    # Leader0 Voting Node data
    tasks: list[str] = LEADER0_NODE_SCHEDULE

    async def wait_for_snapshot(self):
        """Waits for the event snapshot_start time."""
        # get the snapshot start timestamp
        # raises an exception otherwise
        snapshot_start = self.node.get_snapshot_start()
        # check if now is after the snapshot start time
        if not self.node.has_snapshot_started():
            raise Exception(f"snapshot will be stable on {snapshot_start} UTC")

        logger.debug("snapshot is stable")

    async def collect_snapshot_data(self):
        """"""
        # gets the event, raises exception if none is found.
        event = self.node.get_event()
        snapshot_start = self.node.get_snapshot_start()
        if not self.node.has_snapshot_started():
            raise Exception(f"snapshot will be stable on {snapshot_start} UTC")

        # check for this field before getting the data
        is_final = await self.db.check_if_snapshot_is_final(event.row_id)
        if not is_final:
            raise Exception("snapshot is not yet final")

        try:
            # fetch the stable snapshot data
            snapshot = await self.db.fetch_snapshot(event.row_id)
            logger.debug(f"snapshot:\n{snapshot}")
        except Exception as e:
            logger.error(f"expected snapshot:\n{e}")

            # gets event vote plans, raises exception if none is found.
        try:
            voteplans = await self.db.fetch_voteplans(event.row_id)
            logger.debug(f"voteplans:\n{voteplans}")
        except Exception as e:
            logger.error(f"expected voteplans:\n{e}")

        try:
            # gets event proposals, raises exception if none is found.
            proposals = await self.db.fetch_proposals(event.row_id)
            logger.debug(f"proposals:\n{proposals}")
            self.proposals = proposals
        except Exception as e:
            raise Exception(f"failed to fetch proposals from DB: {e}") from e

    async def setup_block0(self):
        """Checks DB event for block0 information, creates and adds it to the DB
        when the needed information is found. Resets the schedule otherwise.
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

            committee_ids = [
                await self.jcli().create_committee_id()
                for _ in range(event.committee_size)
            ]

            # generate genesis file to make block0
            logger.debug("generating genesis content")
            genesis = utils.make_genesis_content(event, leaders, committee_ids)
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
            logger.debug(f"block0 created and saved: {self.node.block0.hash}")

    async def publish_block0(self):
        event = self.node.get_event()
        match self.node.block0:
            case Block0(bin=block0_bytes, hash=block0_hash):
                # push block0 to event table
                await self.db.insert_block0_info(
                    event.row_id, block0_bytes, block0_hash
                )
                # if all is good, we reset the schedule
                logger.debug("inserted block0 info")
            case None:
                logger.error("expected block0 info")
                self.reset_schedule("node has no block0")


class FollowerSchedule(NodeTaskSchedule):
    tasks = FOLLOWER_NODE_SCHEDULE

    def __init__(self, db_url: str, settings: ServiceSettings) -> None:
        NodeTaskSchedule.__init__(self, db_url, settings)
        self.tasks: list[str] = ["todo"]

    async def todo(self):
        pass
