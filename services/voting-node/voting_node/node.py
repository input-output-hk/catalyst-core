import asyncio
import socket
import uvicorn
from typing import Final, List, Optional

from . import logs, tasks, utils
from .config import JormConfig


class VotingNode(uvicorn.Server):
    def __init__(
        self, api_config: uvicorn.Config, jorm_config: JormConfig, database_url: str
    ):
        # initialize uvicorn
        uvicorn.Server.__init__(self, api_config)
        # get logger, this goes away with opentelemetry
        self.logger = logs.getLogger(api_config.log_level)
        # flag that tells the voting node to whether to continue
        # running the task schedule
        self.keep_running = True
        # jorm node params
        self.jorm_config = jorm_config
        # url for database connection
        self.db_url = database_url

    # Use this to run your voting node
    def start(self, sockets: Optional[List[socket.socket]] = None):
        """This method starts the voting node service in an asynchronous runtime. It
        accepts the optional arguments of `sockets` used by the uvicorn server used
        to run the FastAPI service."""
        asyncio.run(self.start_service(sockets=sockets))

    # Starts Voting Node Service, including this fastAPI server as well as the
    # jormungandr node's REST and GRPC servers.
    async def start_service(self, sockets: Optional[List[socket.socket]] = None):
        """Starts Voting Node Service."""
        # time in seconds to wait before retrying to run a schedule
        SLEEP_TO_SCHEDULE_RETRY: Final = 5

        # this is the main task, which stops other tasks by calling the
        # 'stop_schedule' method.
        api_task: asyncio.Task[None] = asyncio.create_task(
            # start the API
            self.start_api(sockets=sockets)
        )

        # checks if `stop_schedule` has been called
        while self.is_running_schedule():
            try:
                # execute the scheduled tasks for this node, by
                # extracting the leadership role from the hostname
                schedule = self.get_schedule()
                if schedule is None:
                    raise Exception("no proper schedule found for this node")
                await schedule.run()
                break
            except Exception as e:
                print(f"schedule failed: {e}")
            # waits before retrying
            await asyncio.sleep(SLEEP_TO_SCHEDULE_RETRY)

        # await the api task until last
        await api_task
        print("Bye bye!")

    async def start_api(self, sockets: Optional[List[socket.socket]] = None):
        """Starts API server for the Voting Node."""
        print("starting api")
        # runs 'serve' method from uvicorn.Server
        await self.serve(sockets=sockets)
        # stops trying to launch jormungandr after API service is finished
        self.stop_schedule()

    def is_running_schedule(self) -> bool:
        return self.keep_running

    def stop_schedule(self):
        self.keep_running = False

    def get_schedule(self):
        # checks the hostname and returns the schedule
        # according to its leadership role.
        match utils.get_leadership_role_by_hostname():
            case "leader0":
                return tasks.Leader0Schedule(self.db_url, self.jorm_config)
            case "leader":
                return tasks.LeaderSchedule(self.db_url, self.jorm_config)
            case "follower":
                return tasks.FollowerSchedule(self.db_url, self.jorm_config)

    async def start_jormungandr(self):
        try:
            await self.jormungandr_subprocess_exec()
        except Exception as e:
            f"jorm error: {e}"
            raise e

    # keeps on launching jormungandr until `stop_schedule()` is called
    async def run_jorm_node(self):
        while self.is_running_schedule():
            jorm_task = asyncio.create_task(self.start_jormungandr())
            try:
                self.logger.debug("jorm task starting")
                await jorm_task
                self.logger.debug("jorm task is finished")
            except Exception as e:
                self.logger.debug(f"jorm failed to start: {e}")
            await asyncio.sleep(1)

    def jormungandr_exec(self) -> str:
        return self.jorm_config.jormungandr_path

    def jcli_exec(self) -> str:
        return self.jorm_config.jcli_path

    async def jormungandr_subprocess_exec(self):
        try:
            proc = await asyncio.create_subprocess_exec(
                f"{self.jormungandr_exec()}",  # "--help",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await proc.communicate()

            if stdout:
                self.logger.info(f"[stdout]\n{stdout.decode()}")
            if stderr:
                self.logger.warning(f"[stderr]\n{stderr.decode()}")

            if proc.returncode != 0:
                raise Exception(
                    f"jormungandr exited with non-zero status: {proc.returncode}"
                )
        except Exception as e:
            self.logger.warning(f"jorm node error: {e}")
            raise e
