import yaml

from pathlib import Path
from typing import Dict, List, NoReturn, Optional

from voting_node.nodes import NodeSettings, PeerNode

from . import logs, utils
from .db import ElectionDb
from .config import NODE_CONFIG_TEMPLATE, JormConfig


# gets voting node logger
logger = logs.getLogger()


class ScheduleRunner(object):
    current_task: Optional[str] = None
    tasks: list[str] = []

    def reset_schedule(self) -> NoReturn:
        """Reset the schedule by setting the current task to None, and raising
        an exception. This method never returns."""
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
    db: ElectionDb
    # storage
    storage: Path

    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        self.settings = NodeSettings(db_url, jorm_config.jcli_path, jorm_config.rest_port, jorm_config.jrpc_port, jorm_config.p2p_port)
        self.db = ElectionDb(db_url)
        self.storage = Path(jorm_config.storage)
        # initialize the dir in case it doesn't exist
        self.storage.mkdir(parents=True, exist_ok=True)
        self.tasks = [
            "bootstrap_db",
            "bootstrap_host",
            "set_node_secrets",
            "set_node_config",
            "fetch_upcoming_election",
            "cleanup",
        ]

    async def bootstrap_db(self):
        # connect to the db
        await self.db.connect()

    async def bootstrap_host(self):
        # gets host information
        try:
            result = await self.db.fetch_leader_node_info()
            logger.debug(f"leader node info retrieved from db: {result}")
            self.node_info = result
        except Exception as e:
            logger.warning(f"leader node info was not fetched: {e}")
            # we add the row
            #  - by adding the keys
            await self.db.insert_leader_node_info(self.settings.jcli_path)
            # if all is good, we save and reset the schedule
            logger.debug("inserted leader node info, resetting the schedule")
            self.reset_schedule()

    async def set_node_secrets(self):
        # node network topology key
        node_topology_key_file = self.storage.joinpath("node_topology_key")
        netkey = self.node_info.netkey
        node_topology_key_file.open("w").write(netkey)
        self.node_topology_key_file = f"{node_topology_key_file}"

        # node secret
        node_secret_file = self.storage.joinpath("node_secret.yaml")
        node_secret = {"bft": {"signing_key": self.node_info.seckey}}
        node_secret_yaml = yaml.dump(node_secret)
        node_secret_file.open("w").write(node_secret_yaml)
        self.node_secret_file = f"{node_secret_file}"

    async def set_node_config(self):
        # node config
        node_config_file = self.storage.joinpath("node_config.yaml")
        node_config = yaml.safe_load(NODE_CONFIG_TEMPLATE)
        host_ip = utils.get_hostname_addr()
        logger.debug(f"host ip: {host_ip}")
        rest_port = self.settings.rest_port
        jrpc_port = self.settings.jrpc_port
        p2p_port = self.settings.p2p_port
        node_config["storage"] = f"{self.storage}"
        node_config["rest"]["listen"] = f"{host_ip}:{rest_port}"
        node_config["jrpc"]["listen"] = f"{host_ip}:{jrpc_port}"
        node_config["p2p"]["bootstrap"]["node_key_file"] = self.node_topology_key_file
        node_config["p2p"]["connection"][
            "public_address"
        ] = f"/ip4/{host_ip}/tcp/{p2p_port}"
        node_config_yaml = yaml.dump(node_config)
        node_config_file.open("w").write(node_config_yaml)
        logger.debug(f"node config: {node_config_yaml}")
        self.node_config_file = f"{node_config_file}"

    async def fetch_upcoming_election(self):
        # This all starts by getting the election row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        try:
            row = await self.db.fetch_upcoming_election()
            logger.debug("current election retrieved from db")
            self.election = row
        except Exception as e:
            raise Exception(f"failed to fetch election from db: {e}") from e

    async def cleanup(self):
        # close the db connection
        await self.db.close()


class LeaderSchedule(NodeTaskSchedule):
    peer_info: Optional[List[PeerNode]]

    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        NodeTaskSchedule.__init__(self, db_url, jorm_config)
        self.tasks: list[str] = [
            "bootstrap_db",
            "bootstrap_host",
            "fetch_peer_info",
            "set_node_secrets",
            "set_node_config",
            "fetch_upcoming_election",
            "get_block0",
            "wait_for_voting",
            "voting",
            "tally",
            "cleanup",
        ]

    async def fetch_peer_info(self):
        # gets info for other leaders
        try:
            # todo
            result = await self.db.fetch_peers_node_info()
            logger.debug(f"saving node info for {len(result)} peers")
            # logger.debug("peer leader node info retrieved from db")
            self.peer_info = result
        except Exception as e:
            logger.warning(f"peer node info not fetched: {e}")
            self.peer_info = None

    async def get_block0(self):
        pass

    async def wait_for_voting(self):
        pass

    async def voting(self):
        pass

    async def tally(self):
        pass


class Leader0Schedule(LeaderSchedule):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        LeaderSchedule.__init__(self, db_url, jorm_config)
        self.tasks: list[str] = [
            "bootstrap_db",
            "bootstrap_host",
            "fetch_peer_info",
            "set_node_secrets",
            "set_node_config",
            "fetch_upcoming_election",
            "fetch_voteplan_proposals",
            "generate_block0",
            "generate_voteplan",
            "wait_for_voting",
            "voting",
            "tally",
            "cleanup",
        ]

    async def fetch_voteplan_proposals(self):
        raise Exception("TODO")

    async def generate_block0(self):
        pass

    async def generate_voteplan(self):
        pass


class FollowerSchedule(NodeTaskSchedule):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        NodeTaskSchedule.__init__(self, db_url, jorm_config)
        self.tasks: list[str] = ["todo"]

    async def todo(self):
        pass
