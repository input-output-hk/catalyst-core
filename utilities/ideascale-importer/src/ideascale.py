import aiohttp
import asyncio
import json
import marshmallow
import marshmallow_dataclass
import rich.progress
from typing import Any, Iterable, List, Mapping

import utils


class BadResponse(Exception):
    def __init__(self):
        super().__init__("Bad response")


class GetFailed(Exception):
    def __init__(self, status, reason, content):
        super().__init__(f"{status} {reason}\n{content})")


class ExcludeUnknownFields:
    class Meta:
        unknown = marshmallow.EXCLUDE


class Campaign(ExcludeUnknownFields):
    """
    Represents a campaign from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    id: int
    name: str
    description: str
    funnel_id: int
    tagline: str
    campaign_url: str


class CampaignGroup(ExcludeUnknownFields):
    """
    Represents a campaign group from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    id: int
    name: str
    campaigns: List[Campaign]


class IdeaAuthorInfo(ExcludeUnknownFields):
    """
    Represents an author info from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    name: str


class Idea(ExcludeUnknownFields):
    """
    Represents an idea from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    id: int
    campaign_id: int
    title: str
    text: str
    author_info: IdeaAuthorInfo
    contributors: List[IdeaAuthorInfo]
    custom_fields_by_key: Mapping[str, str]
    url: str

    def contributors_name(self) -> List[str]:
        return list(map(lambda c: c.name, self.contributors))


class Stage(ExcludeUnknownFields):
    """
    Represents a stage from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    id: int
    key: str
    label: str
    funnel_name: str


class Funnel(ExcludeUnknownFields):
    """
    Represents a funnel from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    id: int
    name: str
    stages: List[Stage]


CampaignSchema = marshmallow_dataclass.class_schema(Campaign)
CampaignGroupSchema = marshmallow_dataclass.class_schema(CampaignGroup)
IdeaAuthorInfoSchema = marshmallow_dataclass.class_schema(IdeaAuthorInfo)
IdeaSchema = marshmallow_dataclass.class_schema(Idea)
StageSchema = marshmallow_dataclass.class_schema(Stage)
FunnelSchema = marshmallow_dataclass.class_schema(Funnel)


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


class Client:
    """
    IdeaScale API client.
    """

    API_URL = "https://cardano.ideascale.com/a/rest"

    def __init__(self, api_token: str):
        self.api_token = api_token
        self.request_counter = 0
        self.request_progress_observer = RequestProgressObserver()

    async def campaigns(self, group_id: int) -> List[Campaign]:
        """
        Gets all campaigns from the campaign group with the given id.
        """

        res = await self._get(f"/v1/campaigns/groups/{group_id}")

        campaigns: List[Campaign] = []
        for group in res:
            assert isinstance(group, dict)

            if "campaigns" in group:
                campaigns.extend(CampaignSchema().load(group["campaigns"], many=True) or [])

        return campaigns

    async def campaign_groups(self) -> List[CampaignGroup]:
        """
        Gets all campaign groups.
        """

        res = await self._get("/v1/campaigns/groups")
        return CampaignGroupSchema().load(res, many=True) or []

    async def campaign_ideas(self, campaign_id: int) -> List[Idea]:
        """
        Gets all ideas from the campaign with the given id.
        """

        res = await self._get(f"/v1/campaigns/{campaign_id}/ideas")
        return IdeaSchema().load(res, many=True) or []

    async def stage_ideas(self, stage_id: int, page_size: int = 50, request_workers_count: int = 10) -> List[Idea]:
        """
        Gets all ideas from the stage with the given id.

        Pages are requested concurrently until the latest one fails
        which signals that that are no more pages left.
        """

        class WorkerData:
            page: int = 0
            done: bool = False
            ideas: List[Idea] = []

        async def worker(d: WorkerData):
            while True:
                if d.done:
                    break

                p = d.page
                d.page += 1

                res = await self._get(f"/v1/stages/{stage_id}/ideas/{p}/{page_size}")
                res_ideas = IdeaSchema().load(res, many=True) or []

                d.ideas.extend(res_ideas)

                if len(res_ideas) < page_size:
                    d.done = True

        d = WorkerData()
        worker_tasks = [asyncio.create_task(worker(d)) for _ in range(request_workers_count)]
        for task in worker_tasks:
            await task

        return d.ideas

    async def campaign_group_ideas(self, group_id: int) -> List[Idea]:
        """
        Gets all ideas from the campaigns that belong to the campaign group with the given id.
        """

        campaigns = await self.campaigns(group_id)
        ideas = await asyncio.gather(*[self.campaign_ideas(c.id) for c in campaigns])
        return [i for campaign_ideas in ideas for i in campaign_ideas]

    async def funnel(self, funnel_id: int) -> Funnel:
        """
        Gets the funnel with the given id.
        """

        funnel = FunnelSchema().load(await self._get(f"/v1/funnels/{funnel_id}"))
        if isinstance(funnel, Funnel):
            return funnel
        else:
            raise BadResponse()

    async def _get(self, path: str) -> Mapping[str, Any] | Iterable[Mapping[str, Any]]:
        """
        Executes a GET request on IdeaScale API.
        """

        url = f"{Client.API_URL}{path}"
        headers = {"api_token": self.api_token}

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
                    utils.snake_case_keys(parsed_json)
                    return parsed_json
                else:
                    raise GetFailed(r.status, r.reason, content)
