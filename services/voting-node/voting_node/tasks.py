from pathlib import Path
import asyncpg
import socket

from voting_node.jcli import JCli

from . import utils
from .config import JormConfig


# task lists are the things that schedules go through
class TaskList(object):
    """A list of task names with corresponding method names that are executed
    sequentially. If the current task raises an exception, running the task list
    again will resume from it."""

    tasks: list[str] = []
    current_task: str | None = None

    def set_reset(self):
        self.current_task = None
        raise Exception("schedule was reset")

    async def run(self) -> None:
        """Runs through the startup tasks. This schedule is called each time that
        a nodes starts."""
        # checks if it should resume from a current task or go through all
        if self.current_task is None:
            tasks = self.tasks[:]
        else:
            task_idx = self.tasks.index(self.current_task)
            tasks = self.tasks[task_idx:]

        for task in tasks:
            try:
                await self.run_task(task)
            except Exception as e:
                raise e

    # runs tasks that are meant to be implemented by deriving classes
    async def run_task(self, task_name):
        try:
            self.current_task = task_name
            print(f"|'{self.__class__.__name__}:{task_name}' is starting")
            task_exec = getattr(self, task_name)
            await task_exec()
            print(f"|'{task_name}' finished")
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
            "load_node_secrets",
            "load_node_config",
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
            print(f"LEADER0 NODE INFO {result}")
            self.node_info = result
        except:
            # we add the row
            #  - by adding the keys
            result = await utils.insert_leader0_node_info(
                self.conn, self.jorm_config.jcli_path
            )
            # if all is good, we save and reset the schedule
            print(f"INSERTED LEADER0 NODE INFO {result}")
            self.set_reset()
    async def set_upcoming_election(self):
        # This all starts by getting the election row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        try:
            row = await utils.fetch_election(self.conn)
            print(f"current election: {row}")
            self.election = row
        except Exception as e:
            raise Exception(f"failed to fetch election from db: {e}")

    async def load_node_secrets(self):
        # Loads keys from storage
        # node network secret key
        node_topology_key = Path(self.jorm_config.storage, "node_topology_key")
        await utils.get_network_secret(node_topology_key, self.jorm_config.jcli_path)

        # node secret file
        node_secret_file = Path(self.jorm_config.storage, "node_secret.yaml")
        if node_secret_file.exists():
            # TODO: need to parse file and extract secret"
            key = node_secret_file.open("r").readline()
            print(f"found key: {key} stored in {node_secret_file.absolute()}")
            self.secret_key = key
            self.secret_key_file = node_secret_file
        else:
            try:
                # run jcli to generate the secret key
                jcli_exec = JCli(self.jorm_config.jcli_path)
                secret = await jcli_exec.seckey("ed25519")
                # write the key to the file in yaml format
                node_secret_file.open("w").write(secret)
                # TODO: need to parse file and extract secret"
                # save the key and the path to the file
                self.secret_key = secret
                self.secret_key_file = node_secret_file
            except Exception as e:
                raise e

    async def load_node_config(self):
        self.set_reset()

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
