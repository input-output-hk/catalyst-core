"""Utility functions and classes."""

from dataclasses import dataclass
from datetime import datetime
import sys
import aiohttp
import asyncio
import json
import csv
from loguru import logger
import re
from typing import Any, Dict, Iterable, List, TypeVar, TYPE_CHECKING

from .db.models import Model

DictOrList = TypeVar("DictOrList", Dict[str, Any], List[Any])


def snake_case_keys(x: DictOrList):
    """Recursively transforms all dict keys to snake_case."""
    if isinstance(x, dict):
        keys = list(x.keys())
        for k in keys:
            v = x.pop(k)
            snake_case_keys(v)
            x[snake_case(k)] = v
    elif isinstance(x, list):
        for i in range(len(x)):
            snake_case_keys(x[i])


def snake_case(s: str) -> str:
    """Transform a string to snake_case."""
    return re.sub(r"([a-z])([A-Z])", r"\1_\2", s).lower()


class RunCmdFailed(Exception):
    """Raised when a command fails to run."""

    def __init__(self, cmd_name: str, exit_code: int, stdout: bytes, stderr: bytes):
        """Initialize a new instance of RunCmdFailed."""
        self.cmd_name = cmd_name
        self.exit_code = exit_code
        self.stdout = stdout
        self.stderr = stderr

    def __str__(self):
        """Return a string representation of the exception."""
        stdout_str = ""
        if len(self.stdout) > 0:
            stdout_str = f"STDOUT:\n{self.stdout.decode()}\n"
        stderr_str = ""
        if len(self.stderr) > 0:
            stderr_str = f"STDERR:\n{self.stderr.decode()}\n"

        lines = [f"Failed to run {self.cmd_name} exit_code={self.exit_code}", stdout_str, stderr_str]

        return "\n".join(lines)


async def run_cmd(name: str, cmd: str):
    """Run a command."""
    with logger.contextualize(name=name):
        logger.info("Executing command", command_line=cmd)
        p = await asyncio.create_subprocess_shell(
            cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            stdin=asyncio.subprocess.PIPE,
        )

        (stdout, stderr) = await p.communicate()
        if p.returncode is not None and p.returncode != 0:
            raise RunCmdFailed(cmd_name=name, exit_code=p.returncode, stdout=stdout, stderr=stderr)
        else:
            logger.info("Successfully ran command")


@dataclass
class RequestProgressInfo:
    """Information about a request's progress."""

    method: str
    url: str
    bytes_received: int
    last_update: datetime


class RequestProgressObserver:
    """Observer used for displaying IdeaScale client's requests progresses."""

    def __init__(self):
        """Initialize a new instance of RequestProgressObserver."""
        self.inflight_requests: Dict[int, RequestProgressInfo] = {}

    def request_start(self, req_id: int, method: str, url: str):
        """Register the start of a request."""
        logger.info("Request started", req_id=req_id, method=method, url=url)
        self.inflight_requests[req_id] = RequestProgressInfo(method, url, 0, datetime.now())

    def request_progress(self, req_id: int, bytes_received: int):
        """Register the progress of a request."""
        info = self.inflight_requests[req_id]
        info.bytes_received += bytes_received

        now = datetime.now()
        if (now - info.last_update).total_seconds() > 1:
            logger.debug(
                "Request progress",
                req_id=req_id,
                method=info.method,
                url=info.url,
                bytes_received=info.bytes_received,
            )
        info.last_update = now

    def request_end(self, req_id):
        """Register the end of a request."""
        info = self.inflight_requests[req_id]
        logger.info(
            "Request finished",
            req_id=req_id,
            method=info.method,
            url=info.url,
            bytes_received=info.bytes_received,
        )
        del self.inflight_requests[req_id]


class BadResponse(Exception):
    """Raised when a request returns a bad response."""

    def __init__(self):
        """Initialize a new instance of BadResponse."""
        super().__init__("Bad response")


class GetFailed(Exception):
    """Raised when a GET request fails."""

    def __init__(self, status: int, content: str):
        """Initialize a new instance of GetFailed."""
        super().__init__(f"Get request failed status={status} content={content}")
        self.status = status
        self.content = content


class HttpClient:
    """HTTP Client for APIs."""

    def __init__(self, api_url: str):
        """Initialize a new instance of HttpClient."""
        self.api_url = api_url
        self.request_progress_observer = RequestProgressObserver()
        self.request_counter = 0
        self.session = aiohttp.ClientSession()

    async def close(self):
        await self.session.close()

    async def json_get(self, path: str, headers: Dict[str, str] = {}) -> Dict[str, Any] | Iterable[Dict[str, Any]]:
        """Execute a GET request and returns a JSON result."""
        content = await self.get(path, headers)
        # Doing this so we can describe schemas with types and
        # not worry about field names not being in snake case format.
        parsed_json = json.loads(content)
        snake_case_keys(parsed_json)
        return parsed_json

    async def get(self, path: str, headers: Dict[str, str] = {}) -> Dict[str, Any] | Iterable[Dict[str, Any]]:
        """Execute a GET request"""
        api_url = self.api_url
        if api_url.endswith("/"):
            api_url = api_url[:-1]

        if not path.startswith("/"):
            path = "/" + path

        url = f"{api_url}{path}"

        # Store request id
        self.request_counter += 1
        req_id = self.request_counter

        self.request_progress_observer.request_start(req_id, "GET", url)
        async with self.session.get(url, headers=headers) as r:
            content = b""

            async for c, _ in r.content.iter_chunks():
                content += c
                self.request_progress_observer.request_progress(req_id, len(c))

            self.request_progress_observer.request_end(req_id)

            if r.status == 200:
                return content
            else:
                raise GetFailed(r.status, content.decode())

    async def post(self, path: str, data: Dict[str, str] = {}, headers: Dict[str, str] = {}) -> Dict[str, Any] | Iterable[Dict[str, Any]]:
        """Execute a POST request."""
        api_url = self.api_url
        if api_url.endswith("/"):
            api_url = api_url[:-1]

        if not path.startswith("/"):
            path = "/" + path

        url = f"{api_url}{path}"

        # Store request id
        self.request_counter += 1
        req_id = self.request_counter

        self.request_progress_observer.request_start(req_id, "GET", url)
        async with self.session.post(url, data = data, headers=headers) as r:
            content = b""

            async for c, _ in r.content.iter_chunks():
                content += c
                self.request_progress_observer.request_progress(req_id, len(c))

            self.request_progress_observer.request_end(req_id)

            if r.status == 200:
                return content
            else:
                raise GetFailed(r.status, content.decode())


log_level_colors = {
    "DEBUG": "blue",
    "INFO": "green",
    "WARNING": "yellow",
    "ERROR": "red",
}


if TYPE_CHECKING:
    from loguru import Record


def logger_formatter(record: "Record") -> str:
    """Formatter for Loguru logger."""
    s = ""
    for k, v in record["extra"].items():
        s += f"<yellow>{k}</yellow>=<cyan>{json.dumps(v)}</cyan> "

    level_c = log_level_colors[record["level"].name]

    return f"{{time}} <{level_c}>{{level: <8}}</{level_c}> {{message}} {s.strip()}\n"


def json_logger_formatter(record: "Record") -> str:
    """JSON Formatter for Loguru logger."""
    record["extra"]["__json_serialized"] = json.dumps(
        {
            "timestamp": record["time"].timestamp(),
            "level": record["level"].name,
            "message": record["message"],
            "source": f"{record['file'].path}:{record['line']}",
            "extra": record["extra"],
        }
    )

    return "{extra[__json_serialized]}\n"


def configure_logger(log_level: str, log_format: str):
    """Configure Loguru logger."""
    formatter = logger_formatter
    if log_format == "json":
        formatter = json_logger_formatter

    logger.remove()
    logger.add(sys.stdout, level=log_level.upper(), format=formatter, enqueue=True)
