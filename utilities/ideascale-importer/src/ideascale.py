import aiohttp
import asyncio
import json
import marshmallow
import marshmallow_dataclass
import rich.progress
from typing import Any, Iterable, List, Mapping, Optional

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
    id: int
    name: str

class CampaignGroup(ExcludeUnknownFields):
    id: int
    name: str
    campaigns: List[Campaign]

class Idea(ExcludeUnknownFields):
    id: int
    title: str
    custom_fields: Optional[Mapping[str, str]]

CampaignSchema = marshmallow_dataclass.class_schema(Campaign)
CampaignGroupSchema = marshmallow_dataclass.class_schema(CampaignGroup)
IdeaSchema = marshmallow_dataclass.class_schema(Idea)

class RequestProgressObserver:
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

class IdeaScale:
    API_URL = "https://cardano.ideascale.com/a/rest"

    def __init__(self, api_token: str):
        self.api_token = api_token
        self.request_counter = 0
        self.request_progress_observer = RequestProgressObserver()

    async def campaigns(self, group_id: int) -> List[Campaign]:
        res = await self._get(f"/v1/campaigns/groups/{group_id}")

        campaigns = []
        for group in res:
            assert isinstance(group, dict)

            if "campaigns" in group:
                campaigns.extend(CampaignSchema().load(group["campaigns"], many=True) or [])

        return campaigns

    async def campaign_groups(self) -> List[CampaignGroup]:
        res = await self._get(f"/v1/campaigns/groups")
        return CampaignGroupSchema().load(res, many=True) or []

    async def campaign_ideas(self, campaign_id: int) -> List[Idea]:
        res = await self._get(f"/v1/campaigns/{campaign_id}/ideas")
        return IdeaSchema().load(res, many=True) or []

    async def campaign_group_ideas(self, group_id: int) -> List[Idea]:
        campaigns = await self.campaigns(group_id)
        ideas = await asyncio.gather(*[self.campaign_ideas(c.id) for c in campaigns])
        return [i for campaign_ideas in ideas for i in campaign_ideas]

    async def _get(self, path: str) -> Mapping[str, Any] | Iterable[Mapping[str, Any]]:
        url = f"{IdeaScale.API_URL}{path}"
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
                    if isinstance(parsed_json, dict):
                        parsed_json = utils.snake_case_keys(parsed_json)
                    elif isinstance(parsed_json, list):
                        for i in range(len(parsed_json)):
                            if isinstance(parsed_json, dict):
                                parsed_json[i] = utils.snake_case_keys(parsed_json[i])

                    return parsed_json
                else:
                    raise GetFailed(r.status, r.reason, content)

def client_with_progress(api_token: str) -> IdeaScale:
    client = IdeaScale(api_token)

    return client
