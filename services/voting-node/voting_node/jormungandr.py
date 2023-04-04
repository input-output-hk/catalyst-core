"""Wrapper for the jormungandr command-line executable."""
import asyncio
from pathlib import Path

from .logs import getLogger

# gets voting node logger
logger = getLogger()


class Jormungandr:
    """Wrapper type for the jormungandr command-line."""

    def __init__(self, jorm_exec: str) -> None:
        self.jorm_exec = jorm_exec

    async def start_leader(self, secret: Path, config: Path, genesis_block: Path):
        try:
            proc = await asyncio.create_subprocess_exec(
                self.jorm_exec,  # "--help",
                "--secret",
                f"{secret}",
                "--config",
                f"{config}",
                "--genesis-block",
                f"{genesis_block}",
                stdout=asyncio.subprocess.PIPE,
            )

            if proc.stdout is not None:
                line = await proc.stdout.readline()
                logger.debug(f"[jorm stdout] {line}")

            if proc.returncode != 0:
                raise Exception(f"jormungandr exited with non-zero status: {proc.returncode}")
        except Exception as e:
            logger.warning(f"jorm node error: {e}")
            raise e
