"""IdeaScale API client."""

import asyncio
import json
from pydantic import BaseModel, Field
from typing import Any, Iterable, List, Mapping

from ideascale_importer import utils
from ideascale_importer.utils import GetFailed


class Campaign(BaseModel):
    """Represents a campaign from IdeaScale.

    (Contains only the fields that are used by the importer).
    """

    id: int
    name: str
    description: str
    funnel_id: int
    tagline: str
    campaign_url: str


class CampaignGroup(BaseModel):
    """Represents a campaign group from IdeaScale.

    (Contains only the fields that are used by the importer).
    """

    id: int
    name: str
    campaigns: List[Campaign]


class IdeaAuthorInfo(BaseModel):
    """Represents an author info from IdeaScale.

    (Contains only the fields that are used by the importer).
    """

    name: str


class Idea(BaseModel):
    """Represents an idea from IdeaScale.

    (Contains only the fields that are used by the importer).
    """

    id: int
    campaign_id: int
    title: str
    text: str
    author_info: IdeaAuthorInfo
    contributors: List[IdeaAuthorInfo]
    url: str
    custom_fields_by_key: Mapping[str, str] = Field(default={})

    def contributors_name(self) -> List[str]:
        """Get the names of all contributors."""
        return list(map(lambda c: c.name, self.contributors))


class Stage(BaseModel):
    """Represents a stage from IdeaScale.

    (Contains only the fields that are used by the importer).
    """

    id: int
    key: str
    label: str
    funnel_name: str


class Funnel(BaseModel):
    """Represents a funnel from IdeaScale.

    (Contains only the fields that are used by the importer).
    """

    id: int
    name: str
    stages: List[Stage]


class StageNotFoundError(Exception):
    def __init__(self, stage_id: int):
        self.stage_id = stage_id

    def __str__(self) -> str:
        return f"No stage found with id {self.stage_id}"


class Client:
    """IdeaScale API client."""

    DEFAULT_API_URL = "https://cardano.ideascale.com"

    def __init__(self, api_token: str, api_url: str = DEFAULT_API_URL):
        """Create an IdeaScale API client which connects to the given API URL."""
        self.api_token = api_token
        self.inner = utils.HttpClient(api_url)

    async def close(self):
        await self.inner.close()

    async def campaigns(self, group_id: int) -> List[Campaign]:
        """Get all campaigns from the campaign group with the given id."""
        res = await self._get(f"/a/rest/v1/campaigns/groups/{group_id}")

        campaigns: List[Campaign] = []
        for group in res:
            assert isinstance(group, dict)

            if "campaigns" in group:
                group_campaigns = []
                for c in group["campaigns"]:
                    group_campaigns.append(Campaign.model_validate(c))
                    await asyncio.sleep(0)

                campaigns.extend(group_campaigns)

        return campaigns

    async def campaign_groups(self) -> List[CampaignGroup]:
        """Get all campaign groups."""
        res = await self._get("/a/rest/v1/campaigns/groups")

        campaign_groups: List[CampaignGroup] = []
        for cg in res:
            campaign_groups.append(CampaignGroup.model_validate(cg))
            await asyncio.sleep(0)

        return campaign_groups

    async def campaign_ideas(self, campaign_id: int) -> List[Idea]:
        """Get all ideas from the campaign with the given id."""
        res = await self._get(f"/a/rest/v1/campaigns/{campaign_id}/ideas")

        ideas = []
        for i in res:
            ideas.append(Idea.model_validate(i))
            await asyncio.sleep(0)

        return ideas

    async def stage_ideas(self, stage_id: int, page_size: int = 50, request_workers_count: int = 10) -> List[Idea]:
        """Get all ideas from the stage with the given id.

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

                res = await self._get(f"/a/rest/v1/stages/{stage_id}/ideas/{p}/{page_size}")

                res_ideas: List[Idea] = []
                for i in res:
                    res_ideas.append(Idea.model_validate(i))

                d.ideas.extend(res_ideas)

                if len(res_ideas) < page_size:
                    d.done = True

        d = WorkerData()

        try:
            await asyncio.create_task(worker(d))
        except GetFailed as e:
            if e.status == 404:
                content = json.loads(e.content)
                if content["key"] == "STAGE_NOT_FOUND":
                    raise StageNotFoundError(stage_id)
                else:
                    raise e
            else:
                raise e

        worker_tasks = [asyncio.create_task(worker(d)) for _ in range(request_workers_count)]
        for task in worker_tasks:
            await task

        return d.ideas

    async def campaign_group_ideas(self, group_id: int) -> List[Idea]:
        """Get all ideas from the campaigns that belong to the campaign group with the given id."""
        campaigns = await self.campaigns(group_id)
        ideas = await asyncio.gather(*[self.campaign_ideas(c.id) for c in campaigns])
        return [i for campaign_ideas in ideas for i in campaign_ideas]

    async def funnel(self, funnel_id: int) -> Funnel:
        """Get the funnel with the given id."""
        res = await self._get(f"/a/rest/v1/funnels/{funnel_id}")
        return Funnel.model_validate(res)

    async def event_themes(self, campaign_id: int, themes_custom_key: str) -> List[str]:
        """Get the list of themes for this Fund,by IdeaScale `campaign_id`."""
        try:
            res = await self._get(f"/a/rest/v1/customFields/idea/campaigns/{campaign_id}")
            themes_fields = [f for f in res if f['key'] and f['key'] == themes_custom_key]
            themes = themes_fields[0]["options"].split('\r\n')
            return themes
        except Exception as e:
            raise Exception(f"Unable to fetch themes: {e}")

    async def _get(self, path: str) -> Mapping[str, Any] | Iterable[Mapping[str, Any]]:
        """Execute a GET request on IdeaScale API."""
        headers = {"api_token": self.api_token}
        return await self.inner.json_get(path, headers)
