import asyncio

from .logs import getLogger

# gets voting node logger
logger = getLogger()


class Jormungandr:
    """Wrapper type for the jormungandr command-line."""

    def __init__(self, jorm_exec: str) -> None:
        self.jorm_exec = jorm_exec

    # keeps on launching jormungandr until `stop_schedule()` is called
    async def run_jorm_node(self):
        jorm_task = asyncio.create_task(self.start_jormungandr())
        try:
            logger.debug("jorm task starting")
            await jorm_task
            logger.debug("jorm task is finished")
        except Exception as e:
            logger.debug(f"jorm failed to start: {e}")

    async def start_jormungandr(self):
        try:
            await self.jormungandr_subprocess_exec()
        except Exception as e:
            f"jorm error: {e}"
            raise e

    async def jormungandr_subprocess_exec(self):
        try:
            proc = await asyncio.create_subprocess_exec(
                self.jorm_exec,  # "--help",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await proc.communicate()

            if stdout:
                logger.info(f"[stdout]\n{stdout.decode()}")
            if stderr:
                logger.warning(f"[stderr]\n{stderr.decode()}")

            if proc.returncode != 0:
                raise Exception(f"jormungandr exited with non-zero status: {proc.returncode}")
        except Exception as e:
            logger.warning(f"jorm node error: {e}")
            raise e
