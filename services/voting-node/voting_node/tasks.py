from pathlib import Path
from typing import NoReturn, Optional

from . import utils
from .db import EventDb
from .config import JormConfig
from .jcli import JCli
from .jormungandr import Jormungandr
from .logs import getLogger
from .models import (
    Block0,
    Block0File,
    GenesisYaml,
    Leader0Node,
    LeaderNode,
    NodeConfigYaml,
    HostInfo,
    NodeSecretYaml,
    NodeSettings,
    NodeTopologyKey,
    VotingNode,
)

# gets voting node logger
logger = getLogger()


class ScheduleRunner(object):
    current_task: Optional[str] = None
    tasks: list[str] = []

    def reset_data(self) -> None:
        """Resets data kept by the schedule runner."""
        self.current_task = None

    def reset_schedule(self, msg: str = "schedule was reset") -> NoReturn:
        """Reset the schedule by setting the current task to None, and raising
        an exception that can be handled by the calling service.

        This method never returns."""
        self.reset_data()
        raise Exception(f"RESET: {msg}")

    async def run(self) -> None:
        """Runs through the scheduled tasks."""
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
                # run the async task
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
    sequentially. If the current task raises an exception, running the task list
    again will resume from it."""

    # node settings
    settings: NodeSettings
    # connection to db
    db: EventDb
    # use JCli to make calls
    jcli_path_str: str
    # use Jormungandr to run the server
    jorm = Jormungandr
    # storage
    storage: Path

    # Voting Node data
    node: VotingNode

    tasks = [
        "connect_db",
        "fetch_upcoming_event",
        "setup_host_info",
        "fetch_leaders",
        "set_node_secret",
        "set_node_topology_key",
        "set_node_config",
        "cleanup",
    ]

    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        self.settings = NodeSettings(
            jorm_config.rest_port,
            jorm_config.jrpc_port,
            jorm_config.p2p_port,
        )
        self.db = EventDb(db_url)
        self.jcli_path_str = jorm_config.jcli_path
        self.jorm = Jormungandr(jorm_config.jorm_path)
        self.storage = Path(jorm_config.storage)
        # initialize the dir in case it doesn't exist
        self.storage.mkdir(parents=True, exist_ok=True)

    # resets data for the node task schedule
    def reset_data(self) -> None:
        ScheduleRunner.reset_data(self)
        self.node.reset()

    async def connect_db(self):
        # connect to the db
        await self.db.connect()

    def jcli(self) -> JCli:
        return JCli(self.jcli_path_str)

    async def fetch_leaders(self):
        # gets info for other leaders
        try:
            # todo
            peers = await self.db.fetch_leaders_host_info()
            logger.debug(f"saving node info for {len(peers)} peers")
            logger.debug(f"peer leader node info retrieved from db {peers}")
            self.node.leaders = peers
        except Exception as e:
            logger.warning(f"peer node info not fetched: {e}")
            self.node.leaders = None

    async def setup_host_info(self):
        # check that we have the info we need, otherwise, we reset
        if self.node.event is None:
            self.reset_schedule("no voting event was found")
        try:
            # gets host information from voting_node table
            event_row_id: int = self.node.event.row_id
            host_info: HostInfo = await self.db.fetch_leader_host_info(event_row_id)
            logger.debug("leader node host info was retrieved from db")
            self.node.host_info = host_info
        except Exception as e:
            # generate and insert host information into voting_node table
            logger.warning(f"leader node info was not fetched: {e}")
            hostname = utils.get_hostname()
            event = self.node.event.row_id
            logger.debug(f"generating {hostname} node info with jcli")
            # generate the keys
            seckey = await self.jcli().privkey(secret_type="ed25519")
            pubkey = await self.jcli().pubkey(seckey)
            netkey = await self.jcli().privkey(secret_type="ed25519")
            logger.debug("node keys were generated")

            host_info = HostInfo(hostname, event, seckey, pubkey, netkey)
            # we add the node info row
            await self.db.insert_leader_host_info(host_info)
            # if all is good, we reset the schedule
            self.reset_schedule("inserted leader node info")

    async def set_node_secret(self):
        # node secret
        match self.node.host_info:
            case HostInfo(_, seckey, _, _):
                node_secret_file = self.storage.joinpath("node_secret.yaml")
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
            case HostInfo(_, _, _, netkey):
                topokey_file = self.storage.joinpath("node_topology_key")
                # save in schedule
                self.node.topology_key = NodeTopologyKey(netkey, topokey_file)
                # write key to file
                self.node.topology_key.save()
            case _:
                self.reset_schedule("no node host info was found")

    async def fetch_upcoming_event(self):
        # This all starts by getting the event row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        try:
            event = await self.db.fetch_upcoming_event()
            logger.debug("current event retrieved from db")
            self.node.event = event
        except Exception as e:
            raise Exception(f"failed to fetch event from db: {e}") from e

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
        listen_rest = f"{host_ip}:{self.settings.rest_port}"
        listen_jrpc = f"{host_ip}:{self.settings.jrpc_port}"
        listen_p2p = f"/ip4/{host_ip}/tcp/{self.settings.p2p_port}"
        trusted_peers = []

        for peer in self.node.leaders:
            match role_n_number:
                case ("leader", 0):
                    pass
                case ("leader", host_number):
                    match utils.get_leadership_role_n_number_by_hostname(peer.hostname):
                        case ("leader", peer_number):
                            if peer_number < host_number:
                                peer_addr = (
                                    f"/ip4/{peer.hostname}/tcp/{self.settings.p2p_port}"
                                )
                                trusted_peers.append({"address": peer_addr})
                        case _:
                            pass
                case ("follower", host_number):
                    peer_addr = f"/ip4/{peer.hostname}/tcp/{self.settings.p2p_port}"
                    trusted_peers.append({"address": peer_addr})

        # node config from default template
        config = utils.make_node_config(
            role_n_number,
            listen_rest,
            listen_jrpc,
            listen_p2p,
            trusted_peers,
            self.storage,
            self.node.topology_key.path,
        )

        # convert to yaml and save
        node_config_yaml = NodeConfigYaml(
            config, self.storage.joinpath("node_config.yaml")
        )
        node_config_yaml.save()
        logger.debug(f"{node_config_yaml}")

    async def cleanup(self):
        # close the db connection
        await self.db.close()


class LeaderSchedule(NodeTaskSchedule):
    # Voting Node data
    node: LeaderNode
    # Tasks for `LeaderNode`
    tasks: list[str] = [
        "connect_db",
        "fetch_upcoming_event",
        "setup_host_info",
        "fetch_leaders",
        "set_node_secret",
        "set_node_topology_key",
        "set_node_config",
        "wait_for_block0",
        "get_block0",
        "wait_for_voting",
        "voting",
        "wait_for_tally",
        "tally",
        "cleanup",
    ]

    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        NodeTaskSchedule.__init__(self, db_url, jorm_config)

    async def wait_for_block0(self):
        pass

    async def get_block0(self):
        # initial checks for data that's needed
        if self.node.leaders is None:
            self.reset_schedule("peer leader info was not found")
        if self.node.event is None:
            self.reset_schedule("event was not found")
        if self.node.event.start_time is None:
            self.reset_schedule("event has no start time")

        # Path to block0.bin file
        block0_path = self.storage.joinpath("block0.bin")
        # Get the optional fields from the current event
        block0 = self.node.event.get_block0()
        # save Block0 to schedule
        self.node.block0 = Block0File(block0, block0_path)
        # write the block bytes to file
        self.node.block0.save()
        logger.debug(f"block0 found in voting event: {self.node.block0}")

    async def wait_for_voting(self):
        pass

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
    tasks: list[str] = [
        "connect_db",
        "fetch_upcoming_event",
        "setup_host_info",
        "fetch_leaders",
        "set_node_secret",
        "set_node_topology_key",
        "set_node_config",
        "wait_for_snapshot",
        "fetch_proposals",
        "setup_block0",
        "publish_block0",
        "wait_for_voting",
        "voting",
        "wait_for_tally",
        "tally",
        "cleanup",
    ]

    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        LeaderSchedule.__init__(self, db_url, jorm_config)

    async def fetch_proposals(self):
        # This all starts by getting the event row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        if self.node.event is None:
            self.reset_schedule("no event was found")
        try:
            proposals = await self.db.fetch_proposals()
            logger.debug(f"proposals:\n{proposals}")
            self.proposals = proposals
        except Exception as e:
            raise Exception(f"failed to fetch proposals from db: {e}") from e

    async def setup_block0(self):
        """Checks DB event for block0 information, creates and adds it to the DB
        when the needed information is found. Resets the schedule otherwise.
        """
        # initial checks for data that's needed
        if self.node.leaders is None:
            self.reset_schedule("peer leader info was not found")
        if self.node.event is None:
            self.reset_schedule("event was not found")

        try:
            # checks for data that's needed
            if self.node.event.block0 is None:
                raise Exception("event has no block0")
            if self.node.event.block0_hash is None:
                raise Exception("event has no block0 hash")
            # Path to block0.bin file
            block0_path = self.storage.joinpath("block0.bin")
            # Get the optional fields from the current event
            block0 = self.node.event.get_block0()
            # save Block0 to schedule
            self.node.block0 = Block0File(block0, block0_path)
            # write the block bytes to file
            self.node.block0.save()
            logger.debug(f"block0 found in voting event: {self.node.block0}")

            # decode genesis and store it after getting block0
            genesis_path = self.storage.joinpath("genesis.yaml")
            block0_path = self.storage.joinpath("block0.bin")
            await self.jcli().decode_block0_bin(block0_path, genesis_path)
            logger.debug(f"decoded and stored file: {genesis_path}")
        except Exception as e:
            logger.debug(f"block0 was not found in voting event: {e}")
            # checks for data that's needed
            if self.node.event.start_time is None:
                self.reset_schedule("event has no start time")

            committee_ids = [
                await self.jcli().create_committee_id()
                for _ in range(self.node.event.committee_size)
            ]

            # generate genesis file to make block0
            logger.debug("generating genesis content")
            genesis = utils.make_genesis_content(
                self.node.event, self.node.leaders, committee_ids
            )
            logger.debug("generated genesis content")
            # convert to yaml and save
            genesis_path = self.storage.joinpath("genesis.yaml")
            genesis_yaml = GenesisYaml(genesis, genesis_path)
            genesis_yaml.save()
            logger.debug(f"{genesis_yaml}")
            self.genesis_yaml = genesis_yaml

            block0_path = self.storage.joinpath("block0.bin")
            await self.jcli().create_block0_bin(block0_path, genesis_path)

            block0_hash = await self.jcli().get_block0_hash(block0_path)
            block0_bytes = block0_path.read_bytes()
            block0 = Block0(block0_bytes, block0_hash)
            self.node.block0 = Block0File(block0, block0_path)
            # write the block bytes to file
            self.node.block0.save()
            logger.debug(f"block0 created and saved: {self.node.block0}")
            # push block0 to event table
            await self.db.insert_block0_info(
                self.node.event.row_id, block0_bytes, block0_hash
            )
            # if all is good, we reset the schedule
            self.reset_schedule("inserted block0 info")

    async def generate_voteplan(self):
        pass


class FollowerSchedule(NodeTaskSchedule):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        NodeTaskSchedule.__init__(self, db_url, jorm_config)
        self.tasks: list[str] = ["todo"]

    async def todo(self):
        pass
