from pathlib import Path
import asyncio
import asyncpg
import datetime
import socket

from .config import JormConfig


# task lists are the things that schedules go through
class TaskList(object):
    """A list of task names with corresponding method names that are executed sequentially.

    If the current task raises an exception, running the task list again will resume from it.

    Raises exceptions"""

    tasks: list[str] = []
    current_task: str | None = None

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
            "get_host_info",
            "get_upcoming_election",
            "load_node_keys",
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

    async def get_upcoming_election(self):
        # This all starts by getting the election row that has the nearest
        # `voting_start`. We query the DB to get the row, and store it.
        query = "SELECT * FROM election WHERE voting_start > $1 ORDER BY voting_start ASC LIMIT 1"
        now = datetime.datetime.utcnow()
        conn = await asyncpg.connect(self.db_url)
        result = await conn.fetchrow(query, now)
        print(f"Result: {result}")
        if result is None:
            raise Exception("no upcoming election found")
        else:
            self.election = result

    async def get_host_info(self):
        # gets host information
        self.hostname = socket.gethostname()
        self.ip_address = socket.gethostbyname(self.hostname)
        print(f"{self.hostname} {self.ip_address}")

    async def load_node_keys(self):
        # Loads keys from storage

        # node topology secret key
        key_file = Path(self.jorm_config.storage, "node_topology_key")
        if key_file.exists():
            key = key_file.open("r").readline()
            print(f"found key: {key}")
            self.topology_key = key
        else:
            try:
                proc = await asyncio.create_subprocess_exec(
                    "jcli",
                    "key",
                    "generate",
                    "--type",
                    "ed25519",
                    stdout=asyncio.subprocess.PIPE,
                )
                data = await proc.stdout.readline()
                key = data.decode().rstrip()
                key_file.open("w").write(key)
                self.topology_key = key
            except Exception as e:
                raise e
        # node topology secret key
        key_file = Path(self.jorm_config.storage, "node_config.yaml")
        if key_file.exists():
            raise Exception("WIP: need to add yaml to the stack")
            print(f"found node: {key}")
            self.secret_key = key
        else:
            try:
                proc = await asyncio.create_subprocess_exec(
                    "jcli",
                    "key",
                    "generate",
                    "--type",
                    "ed25519",
                    stdout=asyncio.subprocess.PIPE,
                )
                data = await proc.stdout.readline()
                key = data.decode().rstrip()
                key_file.open("w").write(key)
                self.secret_key = key
            except Exception as e:
                raise e

    async def load_node_config(self):
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
        pass
