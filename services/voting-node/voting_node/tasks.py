import asyncpg
import yaml

from pathlib import Path
from typing import NoReturn

from . import logs, utils
from .config import JormConfig


# gets voting node logger
logger = logs.getLogger()


# task lists are the things that schedules go through
class TaskList(object):
    """A list of task names with corresponding method names that are executed
    sequentially. If the current task raises an exception, running the task list
    again will resume from it."""

    tasks: list[str] = []
    current_task: str | None = None

    def reset_schedule(self) -> NoReturn:
        """Reset the schedule by setting the current task to None, and raising
        an exception. This method never returns."""
        self.current_task = None
        raise Exception("schedule was reset")

    async def run(self) -> None:
        """Runs through the startup tasks. This schedule is called each time that
        a nodes starts."""
        # checks if it should resume from a current task or go through all
        logger.info("SCHEDULE START")
        if self.current_task is None:
            logger.debug("no current task is set, running all tasks")
            tasks = self.tasks[:]
        else:
            logger.debug(f"'{self.current_task}' is set, resuming")
            task_idx = self.tasks.index(self.current_task)
            tasks = self.tasks[task_idx:]

        for task in tasks:
            try:
                logger.info(f"running '{task}'")
                await self.run_task(task)
            except Exception as e:
                logger.warning(f"'{task}' failed")
                raise e
        logger.info("SCHEDULE STOP")

    # runs tasks that are meant to be implemented by deriving classes
    async def run_task(self, task_name):
        try:
            self.current_task = task_name
            logger.debug(f"|'{self.__class__.__name__}:{task_name}' is starting")
            task_exec = getattr(self, task_name)
            await task_exec()
            logger.debug(f"|'{task_name}' finished")
        except Exception as e:
            raise e


class Leader0Schedule(TaskList):
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
            # "set_node_config",
            # "gather_voteplan_proposal",
            # "generate_block0",
            # "generate_voteplan",
            # "wait_for_voting",
            # "voting",
            # "tally",
            # "cleanup",
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
            raise Exception(f"failed to fetch election from db: {e}")

    async def set_node_secrets(self):
        # node network secret key
        node_topology_key_file = Path(self.jorm_config.storage, "node_topology_key")
        netkey = self.node_info["netkey"]
        node_topology_key_file.open("w").write(netkey)

        # node secret file
        node_secret_file = Path(self.jorm_config.storage, "node_secret.yaml")
        node_secret = { "bft": { "signing_key": self.node_info["seckey"] } }
        node_secret_yaml = yaml.dump(node_secret)
        node_secret_file.open("w").write(node_secret_yaml)

    async def set_node_config(self):
        pass

    async def gather_voteplan_proposal(self):
        pass

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


class LeaderSchedule(TaskList):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        self.db_url = db_url
        self.jorm_config = jorm_config
        self.current_task: str | None = None
        self.tasks: list[str] = ["todo"]

    async def todo(self):
        pass


class FollowerSchedule(TaskList):
    def __init__(self, db_url: str, jorm_config: JormConfig) -> None:
        self.db_url = db_url
        self.jorm_config = jorm_config
        self.current_task: str | None = None
        self.tasks: list[str] = ["todo"]

    async def todo(self):
        pass
