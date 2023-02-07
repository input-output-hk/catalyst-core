import asyncio
from uvicorn import Config, Server
from typing import Union

from fastapi import FastAPI

from .import logs as applogs

app = FastAPI()

@app.get("/")
def heartbeat():
    """Returns 200 if the service is running."""
    return None

class VotingServer(Server):
    def __init__(self, config: Config):
        Server.__init__(self, config)
        self.logger = applogs.getLogger(config.log_level)
        self.retry_jorm = True


    def run(self, jormungandr_path, jcli_path, sockets=None):
        asyncio.run(self.start_service(jormungandr_path, jcli_path, sockets=sockets))

    async def start_service(self, jormungandr_path, jcli_path, sockets=None):
        api_task = asyncio.create_task(self.start_api_server(sockets=sockets))

        jorm_task = asyncio.create_task(self.start_jormungandr(jormungandr_path=jormungandr_path, jcli_path=jcli_path))

        while self.retry_jorm: 
            try:
                self.logger.debug("jorm task starting")
                await jorm_task
                self.logger.debug("jorm task is finished")
                break
            except Exception as e:
                self.logger.debug(f"jorm failed to start: {e}")
                await asyncio.sleep(1)

        print(await api_task)

    async def start_api_server(self, sockets=None):
        print('starting api')
        await self.serve(sockets=sockets)
        self.retry_jorm = False

    async def start_jormungandr(self, jormungandr_path, jcli_path):
        try:
            await self.try_to_start_jormungandr(jormungandr_path, jcli_path)
        except Exception as e:
            f"jorm error: {e}"
            raise e


    async def try_to_start_jormungandr(self, jormungandr_path, jcli_path):
        try:
            proc = await asyncio.create_subprocess_exec(f"{jormungandr_path}", #"--help",
                                                        stdout=asyncio.subprocess.PIPE,
                                                        stderr=asyncio.subprocess.PIPE,
                                                        )
            stdout, stderr = await proc.communicate()

            if stdout:
                self.logger.info(f"[stdout]\n{stdout.decode()}")
            if stderr:
                self.logger.warning(f"[stderr]\n{stderr.decode()}")

            if proc.returncode != 0:
                raise Exception(f"jormungandr exited with non-zero status: {proc.returncode}")
        except Exception as e:
            self.logger.warning(f"jorm node error: {e}")
            raise e

def run(jormungandr_path, jcli_path, host="127.0.0.1", port=8000, log_level="info"):
    """Entrypoint to running the service."""
    config = Config(app=app, host=host, port=port, log_level=log_level)
    server = VotingServer(config=config)
    server.run(jormungandr_path=jormungandr_path, jcli_path=jcli_path)
