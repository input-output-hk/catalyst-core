"""The voting node service.

VotingService.start() initiates the API and starts executing the schedule for the node.
"""
import asyncio
import socket
from typing import Final

import uvicorn
from loguru import logger

from voting_node.models import ServiceSettings

from . import tasks, utils

# time in seconds to wait before retrying to run a schedule
SLEEP_TO_SCHEDULE_RETRY: Final = 6


class VotingService(uvicorn.Server):
    """Voting node as a service."""

    def __init__(self, api_config: uvicorn.Config, settings: ServiceSettings) -> None:
        """Create a voting service instance by initializing the FastAPI server, and storing the node settings."""
        # initialize uvicorn
        uvicorn.Server.__init__(self, api_config)
        # flag that tells the voting node to whether to continue
        # running the task schedule
        self.keep_running = True
        # jorm node params
        self.settings = settings

    # Use this to run your voting node
    def start(self, sockets: list[socket.socket] | None = None):
        """Start the voting node service in an asynchronous runtime.

        It accepts the optional arguments of `sockets` used by the uvicorn server used to run the FastAPI service.
        """
        try:
            asyncio.run(self.start_service(sockets=sockets))
        except Exception as e:
            logger.error(f"Schedule error: {e}")
            print("Schedule failed. Exiting.")

    # Starts Voting Node Service, including this fastAPI server as well as the
    # jormungandr node's REST and GRPC servers.
    async def start_service(self, sockets: list[socket.socket] | None = None):
        """Start Voting Node Service."""
        # this is the main task, which stops other tasks by calling the
        # 'stop_schedule' method.
        api_task: asyncio.Task[None] = asyncio.create_task(
            # start the API
            self.start_api(sockets=sockets),
        )

        # wait for the stdout from uvicorn before our logs start to roll
        await asyncio.sleep(1)
        # execute the scheduled tasks for this node, by
        # extracting the leadership role from the hostname
        schedule = self.get_schedule()
        match schedule:
            case None:
                raise Exception("no proper schedule found for this node")
            case _:
                # checks if `stop_schedule` has been called
                while self.is_running_schedule():
                    try:
                        await schedule.run()
                        break
                    except Exception as e:
                        logger.warning(f"X-> RESET: {e}")
                    # waits before retrying
                    await asyncio.sleep(SLEEP_TO_SCHEDULE_RETRY)

        # await the api task until last
        await api_task

        print("Bye bye!")

    async def start_api(self, sockets: list[socket.socket] | None = None):
        """Start API server for the Voting Node."""
        logger.info("starting api")
        # runs 'serve' method from uvicorn.Server
        await self.serve(sockets=sockets)
        # stops trying to launch jormungandr after API service is finished
        self.stop_schedule()

    def is_running_schedule(self) -> bool:
        """Return True if the schedule is running."""
        return self.keep_running

    def stop_schedule(self):
        """Stop the schedule from running."""
        self.keep_running = False

    def get_schedule(self):
        """Get schedule according to the node hostname."""
        # checks the hostname and returns the schedule
        # according to its leadership role.
        # raises exception is something goes wrong with the hostname
        role_str = self.settings.role
        if role_str is None:
            role_str = utils.get_hostname().lower()

        match utils.parse_node_role(role_str):
            case utils.NodeRole("leader", 0):
                return tasks.Leader0Schedule(self.settings)
            case utils.NodeRole("leader", _):
                return tasks.LeaderSchedule(self.settings)
            case utils.NodeRole("follower", _):
                return tasks.FollowerSchedule(self.settings)
            case _:
                return None
