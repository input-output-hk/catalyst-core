import asyncpg
import datetime


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
    def __init__(self, db_pool: asyncpg.Pool | None) -> None:
        self.db_pool = db_pool
        self.current_task: str | None = None
        self.tasks: list[str] = [
            "check_next_voting_start",
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

    async def check_next_voting_start(self):
        raise Exception("todo: wip")

    async def load_node_keys(self):
        pass

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
