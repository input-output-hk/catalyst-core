import asyncio
from prometheus_fastapi_instrumentator import Instrumentator
from uvicorn import Config, Server

from fastapi import FastAPI

from . import logs

app = FastAPI()


@app.get("/")
def heartbeat():
    """Returns 200 if the service is running."""
    return None


Instrumentator().instrument(app).expose(app)


class JormConfig(object):
    def __init__(self, jormungandr_path: str, jcli_path: str):
        self.jormungandr_path = jormungandr_path
        self.jcli_path = jcli_path


class VotingServer(Server):
    def __init__(self, config: Config, jorm_config: JormConfig):
        Server.__init__(self, config)
        self.logger = logs.getLogger(config.log_level)
        self.retry_jorm = True
        self.jorm_config = jorm_config

    def jormungandr_exec(self) -> str:
        return self.jorm_config.jormungandr_path

    def jcli_exec(self) -> str:
        return self.jorm_config.jcli_path

    def run(self, sockets=None):
        asyncio.run(self.start_service(sockets=sockets))

    def keeps_trying(self) -> bool:
        return self.retry_jorm

    def stop_trying(self):
        self.retry_jorm = False

    # Starts Voting Node Service, including this fastAPI server as well as the
    # jormungandr node's REST and GRPC servers.
    async def start_service(self, sockets=None):
        """Starts Voting Node Service."""
        api_task = asyncio.create_task(self.start_api_server(sockets=sockets))

        # keeps on launching jormungandr until `stop_trying()` is called
        while self.keeps_trying():
            jorm_task = asyncio.create_task(self.start_jormungandr())
            try:
                self.logger.debug("jorm task starting")
                await jorm_task
                self.logger.debug("jorm task is finished")
            except Exception as e:
                self.logger.debug(f"jorm failed to start: {e}")
            await asyncio.sleep(1)
        print(await api_task)

    async def start_api_server(self, sockets=None):
        """Starts API server for the Voting Node."""
        print("starting api")
        await self.serve(sockets=sockets)
        # stops trying to launch jormungandr after API service is finished
        self.stop_trying()

    async def start_jormungandr(self):
        try:
            await self.jormungandr_subprocess_exec()
        except Exception as e:
            f"jorm error: {e}"
            raise e

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


# Use this to run your service
def run(jormungandr_path, jcli_path, host="127.0.0.1", port=8000, log_level="info"):
    """Main entrypoint to running the service."""
    api_config = Config(app=app, host=host, port=port, log_level=log_level)
    jorm_config = JormConfig(jormungandr_path=jormungandr_path, jcli_path=jcli_path)
    server = VotingServer(config=api_config, jorm_config=jorm_config)
    server.run()
