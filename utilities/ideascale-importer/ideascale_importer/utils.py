import asyncio
import json
import aiohttp
import rich
import rich.progress
import re
from typing import Any, Dict, Iterable, List, Mapping, TypeVar


DictOrList = TypeVar("DictOrList", Dict[str, Any], List[Any])


def snake_case_keys(x: DictOrList):
    """
    Recursively transforms all dict keys to snake_case.
    """

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
    """
    Transforms a string to snake_case.
    """

    return re.sub(r"([a-z])([A-Z])", r"\1_\2", s).lower()


class RunCmdFailed(Exception):
    def __init__(self, cmd_name: str, exit_code: int, stdout: bytes, stderr: bytes):
        self.cmd_name = cmd_name
        self.exit_code = exit_code
        self.stdout = stdout
        self.stderr = stderr

    def __str__(self):
        stdout_str = ""
        if len(self.stdout) > 0:
            stdout_str = f"STDOUT:\n{self.stdout.decode()}\n"
        stderr_str = ""
        if len(self.stderr) > 0:
            stderr_str = f"STDERR:\n{self.stderr.decode()}\n"

        lines = [f"Failed to run {self.cmd_name} exit_code={self.exit_code}", stdout_str, stderr_str]

        return "\n".join(lines)


async def run_cmd(console: rich.console.Console, name: str, cmd: str):
    console.print(f"Executing {cmd}")
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
        console.print(f"Successfully ran {name}")
        if len(stdout) > 0:
            console.print(f"STDOUT:\n{stdout.decode()}")


class RequestProgressObserver:
    """
    Observer used for displaying IdeaScale client's requests progresses.
    """

    def __init__(self):
        self.inflight_requests = {}
        self.progress = rich.progress.Progress(
            rich.progress.TextColumn("{task.description}"),
            rich.progress.DownloadColumn(),
            rich.progress.TransferSpeedColumn(),
            rich.progress.SpinnerColumn(),
        )

    def request_start(self, req_id: int, method: str, url: str):
        self.inflight_requests[req_id] = [self.progress.add_task(f"({req_id}) {method} {url}", total=None), 0]

    def request_progress(self, req_id: int, total_bytes_received: int):
        self.inflight_requests[req_id][1] = total_bytes_received
        self.progress.update(self.inflight_requests[req_id][0], completed=total_bytes_received)

    def request_end(self, req_id):
        self.progress.update(self.inflight_requests[req_id][0], total=self.inflight_requests[req_id][1])

    def __enter__(self):
        self.progress.__enter__()

    def __exit__(self, *args):
        self.progress.__exit__(*args)

        for [task_id, _] in self.inflight_requests.values():
            self.progress.remove_task(task_id)
        self.inflight_requests.clear()


class BadResponse(Exception):
    def __init__(self):
        super().__init__("Bad response")


class GetFailed(Exception):
    def __init__(self, status, reason, content):
        super().__init__(f"{status} {reason}\n{content})")


class JsonHttpClient:
    def __init__(self, api_url: str):
        self.api_url = api_url
        self.request_progress_observer = RequestProgressObserver()
        self.request_counter = 0

    async def get(self, path: str, headers: Mapping[str, str] = {}) -> Mapping[str, Any] | Iterable[Mapping[str, Any]]:
        """
        Executes a GET request on IdeaScale API.
        """

        api_url = self.api_url
        if api_url.endswith("/"):
            api_url = api_url[:-1]

        if not path.startswith("/"):
            path = "/" + path

        url = f"{api_url}{path}"

        # Store request id
        self.request_counter += 1
        req_id = self.request_counter

        async with aiohttp.ClientSession() as session:
            self.request_progress_observer.request_start(req_id, "GET", url)
            async with session.get(url, headers=headers) as r:
                content = b''

                async for c, _ in r.content.iter_chunks():
                    content += c
                    self.request_progress_observer.request_progress(req_id, len(content))

                self.request_progress_observer.request_end(req_id)

                if r.status == 200:
                    # Doing this so we can describe schemas with types and
                    # not worry about field names not being in snake case format.
                    parsed_json = json.loads(content)
                    snake_case_keys(parsed_json)
                    return parsed_json
                else:
                    raise GetFailed(r.status, r.reason, content)
