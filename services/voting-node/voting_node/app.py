import asyncio
import logging
from uvicorn import Config, Server
from typing import Union

from fastapi import FastAPI
from fastapi.logger import logger as fastapi_logger

app = FastAPI()

@app.get("/")
def heartbeat():
    """Returns 200 if the service is running."""
    return None

class VotingServer(Server):
    def __init__(self, logger, config: Config):
        Server.__init__(self, config)
        self.logger = logger


    def run(self, jormungandr_path, jcli_path, sockets=None):
        asyncio.run(self.start_service(jormungandr_path, jcli_path, sockets=sockets))

    async def start_service(self, jormungandr_path, jcli_path, sockets=None):
        api_task = asyncio.create_task(self.start_api_server(sockets=sockets))

        jorm_task = asyncio.create_task(self.start_jormungandr(jormungandr_path=jormungandr_path, jcli_path=jcli_path))

        try:
            await jorm_task
            self.logger.debug("jorm task is finished")
        except:
            self.logger.debug("jorm failed to start")

        print(await api_task)

    async def start_api_server(self, sockets=None):
        print('starting api')
        await self.serve(sockets=sockets)

    async def start_jormungandr(self, jormungandr_path, jcli_path):
        try:
            try:
                proc = await asyncio.create_subprocess_exec(f"{jormungandr_path}",
                                                            stdout=asyncio.subprocess.PIPE,
                                                            stderr=asyncio.subprocess.PIPE,
                                                            )
                stdout, stderr = await proc.communicate()

                if proc.returncode != 0:
                    raise Exception("jormungandr exited with non-zero status")

                if stdout:
                    self.logger.info(f"[stdout]\n{stdout.decode()}")
                if stderr:
                    self.logger.warning(f"[stderr]\n{stderr.decode()}")
            except Exception as e:
                self.logger.warning(f"jorm node error: {e}")

        except:
            "jorm error"


def run(jormungandr_path, jcli_path, host="127.0.0.1", port=8000, log_level="info"):
    """Entrypoint to running the service."""
    logger = logging.getLogger(__name__)
    logger.setLevel(getattr(logging, log_level.upper()))
    ch = logging.StreamHandler()
    ch.setLevel(getattr(logging, log_level.upper()))
    formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    ch.setFormatter(formatter)
    logger.addHandler(ch)

    config = Config(app=app, host=host, port=port, log_level=log_level)
    server = VotingServer(logger, config=config)
    server.run(jormungandr_path=jormungandr_path, jcli_path=jcli_path)
