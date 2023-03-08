from pathlib import Path
from typing import List, NoReturn, Optional

from . import utils
from .db import EventDb
from .config import JormConfig
from .jcli import JCli
from .jormungandr import Jormungandr
from .logs import getLogger
from .models import (
    Block0,
    Event,
    GenesisYaml,
    NodeConfigYaml,
    NodeInfo,
    NodeSecretYaml,
    NodeSettings,
    NodeTopologyKey,
    PeerNode,
    Proposal,
)

# gets voting node logger
logger = getLogger()


class ScheduleRunner(object):
    current_task: Optional[str] = None
    tasks: list[str] = []

    def reset_schedule(self) -> NoReturn:
        """Reset the schedule by setting the current task to None, and raising
        an exception that can be handled by the calling service.

        This method never returns."""
        self.current_task = None
        raise Exception("schedule was reset")

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
                raise Exception(f"'{task}' error: {e}") from e
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
    node_info: Optional[NodeInfo] = None
    leaders_info: Optional[List[PeerNode]] = None
    node_config: Optional[NodeConfigYaml] = None
    node_secret: Optional[NodeSecretYaml] = None
    topology_key: Optional[NodeTopologyKey] = None
    voting_event: Optional[Event] = None

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
        self.tasks = [
            "connect_db",
            "fetch_upcoming_event",
            "setup_node_info",
            "fetch_leaders_info",
            "set_node_secret",
            "set_node_topology_key",
            "set_node_config",
            "cleanup",
        ]

    async def connect_db(self):
        # connect to the db
        await self.db.connect()

    def jcli(self) -> JCli:
        return JCli(self.jcli_path_str)

    async def fetch_leaders_info(self):
        # gets info for other leaders
        try:
            # todo
            peers = await self.db.fetch_leaders_host_info()
            logger.debug(f"saving node info for {len(peers)} peers")
            logger.debug(f"peer leader node info retrieved from db {peers}")
            self.leaders_info = peers
        except Exception as e:
            logger.warning(f"peer node info not fetched: {e}")
            self.leaders_info = None

    async def setup_node_info(self):
        # check that we have the info we need, otherwise, we reset
        if self.voting_event is None:
            logger.debug("no voting event was found, resetting.")
            self.reset_schedule()
        try:
            # gets host information from voting_node table
            event_row_id: int = self.voting_event.row_id
            node_info: NodeInfo = await self.db.fetch_leader_node_info(event_row_id)
            logger.debug("leader node host info was retrieved from db")
            self.node_info = node_info
        except Exception as e:
            # generate and insert host information into voting_node table
            logger.warning(f"leader node info was not fetched: {e}")
            hostname = utils.get_hostname()
            event = self.voting_event.row_id
            logger.debug(f"generating {hostname} node info with jcli")
            # generate the keys
            seckey = await self.jcli().privkey(secret_type="ed25519")
            pubkey = await self.jcli().pubkey(seckey=seckey)
            netkey = await self.jcli().privkey(secret_type="ed25519")
            logger.debug("node keys were generated")

            node_info = NodeInfo(hostname, event, seckey, pubkey, netkey)
            # we add the node info row
            await self.db.insert_leader_node_info(node_info)
            # if all is good, we reset the schedule
            logger.debug("inserted leader node info, resetting the schedule")
            self.reset_schedule()

    async def set_node_secret(self):
        # node secret
        match self.node_info:
            case NodeInfo(_, seckey, _, _):
                node_secret_file = self.storage.joinpath("node_secret.yaml")
                node_secret = {"bft": {"signing_key": seckey}}
                # save in schedule
                self.node_secret = NodeSecretYaml(node_secret, node_secret_file)
                # write key to file
                self.node_secret.save()
                logger.debug(f"{self.node_secret.path} saved")
            case _:
                logger.debug("no node host info was found, resetting.")
                self.reset_schedule()

    async def set_node_topology_key(self):
        # node network topology key
        match self.node_info:
            case NodeInfo(_, _, _, netkey):
                topokey_file = self.storage.joinpath("node_topology_key")
                # save in schedule
                self.topology_key = NodeTopologyKey(netkey, topokey_file)
                # write key to file
                self.topology_key.save()
            case _:
                logger.debug("no node host info was found, resetting.")
                self.reset_schedule()

    async def fetch_upcoming_event(self):
        # This all starts by getting the event row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        try:
            event = await self.db.fetch_upcoming_event()
            logger.debug("current event retrieved from db")
            self.voting_event = event
        except Exception as e:
            raise Exception(f"failed to fetch event from db: {e}") from e

    async def set_node_config(self):
        # check that we have the info we need, otherwise, we reset
        if self.topology_key is None:
            logger.debug("no node topology key was found, resetting.")
            self.reset_schedule()
        if self.leaders_info is None:
            logger.debug("no leader info was found, resetting.")
            self.reset_schedule()

        #  modify node config for all nodes
        host_name = utils.get_hostname()
        host_ip = utils.get_hostname_addr()
        role_n_number = utils.get_leadership_role_n_number_by_hostname(host_name)
        logger.debug(f"{role_n_number} ip: {host_ip}")
        listen_rest = f"{host_ip}:{self.settings.rest_port}"
        listen_jrpc = f"{host_ip}:{self.settings.jrpc_port}"
        listen_p2p = f"/ip4/{host_ip}/tcp/{self.settings.p2p_port}"
        trusted_peers = []

        for peer in self.leaders_info:
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
            self.topology_key.path,
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
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        NodeTaskSchedule.__init__(self, db_url, jorm_config)
        self.tasks: list[str] = [
            "connect_db",
            "fetch_upcoming_event",
            "setup_node_info",
            "fetch_leaders_info",
            "set_node_secret",
            "set_node_topology_key",
            "set_node_config",
            "get_block0",
            "wait_for_voting",
            "voting",
            "tally",
            "cleanup",
        ]

    async def get_block0(self):
        pass

    async def wait_for_voting(self):
        pass

    async def voting(self):
        pass

    async def tally(self):
        pass


class Leader0Schedule(LeaderSchedule):
    # Leader0 Voting Node data
    block0_bin: Block0
    genesis_yaml: GenesisYaml
    proposals: List[Proposal]

    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        LeaderSchedule.__init__(self, db_url, jorm_config)
        self.tasks: list[str] = [
            "connect_db",
            "fetch_upcoming_event",
            "setup_node_info",
            "fetch_leaders_info",
            "set_node_secret",
            "set_node_topology_key",
            "set_node_config",
            "fetch_proposals",
            "setup_block0",
            "wait_for_voting",
            "voting",
            "tally",
            "cleanup",
        ]

    async def fetch_proposals(self):
        # This all starts by getting the event row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        if self.voting_event is None:
            logger.debug("no event was found, resetting.")
            self.reset_schedule()
        try:
            proposals = await self.db.fetch_proposals()
            logger.debug(f"proposals:\n{proposals}")
            self.proposals = proposals
        except Exception as e:
            raise Exception(f"failed to fetch proposals from db: {e}") from e

    async def setup_block0(self):
        # initial checks for data that's needed
        if self.leaders_info is None:
            logger.debug("peer leader info was not found, resetting.")
            self.reset_schedule()
        if self.voting_event is None or self.voting_event.start_time is None:
            logger.debug("event was not found, resetting.")
            self.reset_schedule()

        if (
            self.voting_event.block0 is not None
            and self.voting_event.block0_hash is not None
        ):
            # fetch block0 from db event
            block0 = self.storage.joinpath("block0.bin")
            block0.write_bytes(self.voting_event.block0)
            block0_hash = await utils.make_block0_hash(self.jcli_path_str, block0)
            self.block0 = Block0(block0, block0_hash)
            logger.debug(f"block0 fetched: {self.block0}")
        else:
            # generate genesis file to make block0
            genesis = utils.make_genesis_file(
                self.voting_event.start_time, self.leaders_info
            )
            logger.debug("generated genesis")
            # convert to yaml and save
            genesis_yaml = GenesisYaml(genesis, self.storage.joinpath("genesis.yaml"))
            genesis_yaml.save()
            logger.debug(f"{genesis_yaml}")
            self.genesis_yaml = genesis_yaml
            block0, block0_hash = await utils.make_block0(
                self.jcli_path_str, self.storage, genesis_yaml.path
            )
            self.block0 = Block0(block0, block0_hash)
            logger.debug(f"block0 created: {self.block0}")
            # push block0 to event table
            block0_bytes = block0.read_bytes()
            await self.db.insert_block0_info(
                self.voting_event.row_id, block0_bytes, block0_hash
            )
            # if all is good, we reset the schedule
            logger.debug("inserted block0 info, resetting the schedule")
            self.reset_schedule()

    async def generate_voteplan(self):
        pass


class FollowerSchedule(NodeTaskSchedule):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        NodeTaskSchedule.__init__(self, db_url, jorm_config)
        self.tasks: list[str] = ["todo"]

    async def todo(self):
        pass
