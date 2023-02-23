import asyncpg
import yaml

from pathlib import Path
from typing import NoReturn

from . import logs, utils
from .config import NODE_CONFIG_TEMPLATE, JormConfig


# gets voting node logger
logger = logs.getLogger()


class TaskSchedule(object):
    """A schedule of task names with corresponding method names that are executed
    sequentially. If the current task raises an exception, running the task list
    again will resume from it."""

    tasks: list[str] = []
    current_task: str | None = None

    def reset_schedule(self) -> NoReturn:
        """Reset the schedule by setting the current task to None, and raising
        an exception. This method never returns."""
        self.current_task = None
        raise Exception("schedule was reset")

    def is_resuming(self) -> bool:
        return self.current_task is not None

    async def run(self) -> None:
        """Runs through the startup tasks. This schedule is called each time that
        a nodes starts."""
        # checks if it should resume from a current task or go through all
        if self.is_resuming():
            task_idx = self.tasks.index(self.current_task)
            tasks = self.tasks[task_idx:]
            logger.info("SCHEDULE RESTART")
        else:
            tasks = self.tasks[:]
            logger.info("SCHEDULE START")

        for task in tasks:
            try:
                # runs tasks that are meant to be implemented by deriving classes
                await self.run_task(task)
            except Exception as e:
                raise Exception(f"'{task}' error: {e}") from e
        logger.info("SCHEDULE END")

    async def run_task(self, task_name):
        logger.info(f"{task_name}")
        logger.debug(f"|'{task_name}' start")
        self.current_task = task_name
        task_exec = getattr(self, task_name)
        await task_exec()
        logger.debug(f"|'{task_name}' end")


class Leader0Schedule(TaskSchedule):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        self.db_url = db_url
        self.jorm_config = jorm_config
        self.current_task: str | None = None
        self.tasks: list[str] = [
            "bootstrap_storage",
            "bootstrap_db",
            "bootstrap_host",
            "set_upcoming_election",
            "set_node_secrets",
            "set_node_config",
            "gather_voteplan_proposal",
            "generate_block0",
            "generate_voteplan",
            "wait_for_voting",
            "voting",
            "tally",
            "cleanup",
        ]

    async def bootstrap_storage(self):
        # finds or tries to create the storage from
        # its path. Raise exception if it can't
        self.storage = Path(self.jorm_config.storage).mkdir(parents=True, exist_ok=True)

    async def bootstrap_db(self):
        conn = await asyncpg.connect(self.db_url)
        if conn is None:
            raise Exception("failed to connect to the database")
        self.conn = conn

    async def bootstrap_host(self):
        # gets host information
        try:
            result = await utils.fetch_leader0_node_info(self.conn)
            logger.debug("leader0 node info retrieved from db")
            self.node_info = result
        except Exception as e:
            logger.warning(f"leader0 node info was not fetched: {e}")
            # we add the row
            #  - by adding the keys
            await utils.insert_leader0_node_info(self.conn, self.jorm_config.jcli_path)
            # if all is good, we save and reset the schedule
            logger.debug("inserted leader0 node info, resetting the schedule")
            self.reset_schedule()

    async def set_upcoming_election(self):
        # This all starts by getting the election row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        try:
            row = await utils.fetch_election(self.conn)
            logger.debug("current election retrieved from db")
            self.election = row
        except Exception as e:
            raise Exception(f"failed to fetch election from db: {e}") from e

    async def set_node_secrets(self):
        # node network topology key
        node_topology_key_file = Path(self.jorm_config.storage, "node_topology_key")
        netkey = self.node_info["netkey"]
        node_topology_key_file.open("w").write(netkey)
        self.node_topology_key_file = f"{node_topology_key_file}"

        # node secret
        node_secret_file = Path(self.jorm_config.storage, "node_secret.yaml")
        node_secret = {"bft": {"signing_key": self.node_info["seckey"]}}
        node_secret_yaml = yaml.dump(node_secret)
        node_secret_file.open("w").write(node_secret_yaml)
        self.node_secret_file = f"{node_secret_file}"

    async def set_node_config(self):
        # node config
        node_config_file = Path(self.jorm_config.storage, "node_config.yaml")
        node_config = yaml.safe_load(NODE_CONFIG_TEMPLATE)
        host_ip = utils.get_hostname_addr()
        logger.debug(f"host ip: {host_ip}")
        rest_port = self.jorm_config.rest_port
        jrpc_port = self.jorm_config.jrpc_port
        p2p_port = self.jorm_config.p2p_port
        node_config["storage"] = self.jorm_config.storage
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

    async def gather_voteplan_proposal(self):
        raise Exception("TODO")

    async def generate_block0(self):
        pass

    async def generate_voteplan(self):
        pass

    async def wait_for_voting(self):
        pass

    async def voting(self):
        pass

    async def tally(self):
        pass

    async def cleanup(self):
        # if the connection to the DB is there, close it.
        if self.conn is not None:
            await self.conn.close()


class LeaderSchedule(TaskSchedule):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        self.db_url = db_url
        self.jorm_config = jorm_config
        self.current_task: str | None = None
        self.tasks: list[str] = ["todo"]

    async def todo(self):
        pass


class FollowerSchedule(TaskSchedule):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        self.db_url = db_url
        self.jorm_config = jorm_config
        self.current_task: str | None = None
        self.tasks: list[str] = ["todo"]

    async def todo(self):
        pass
