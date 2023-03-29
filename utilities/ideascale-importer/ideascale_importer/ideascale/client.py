import asyncio
from pydantic.dataclasses import dataclass
import pydantic.tools
from typing import Any, Iterable, List, Mapping

from ideascale_importer import utils


class ExcludeUnknownFields:
    ...


@dataclass
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


@dataclass
class CampaignGroup(ExcludeUnknownFields):
    """
    Represents a campaign group from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    id: int
    name: str
    campaigns: List[Campaign]


@dataclass
class IdeaAuthorInfo(ExcludeUnknownFields):
    """
    Represents an author info from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    name: str


@dataclass
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


@dataclass
class Stage(ExcludeUnknownFields):
    """
    Represents a stage from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    id: int
    key: str
    label: str
    funnel_name: str


@dataclass
class Funnel(ExcludeUnknownFields):
    """
    Represents a funnel from IdeaScale.
    (Contains only the fields that are used by the importer).
    """

    id: int
    name: str
    stages: List[Stage]


class Client:
    """
    IdeaScale API client.
    """

    API_URL = "https://cardano.ideascale.com/a/rest"

    def __init__(self, api_token: str):
        self.api_token = api_token
        self.inner = utils.JsonHttpClient(Client.API_URL)

    async def campaigns(self, group_id: int) -> List[Campaign]:
        """
        Gets all campaigns from the campaign group with the given id.
        """

        res = await self._get(f"/v1/campaigns/groups/{group_id}")

        campaigns: List[Campaign] = []
        for group in res:
            assert isinstance(group, dict)

            if "campaigns" in group:
                group_campaigns = []
                for c in group["campaigns"]:
                    pydantic.tools.parse_obj_as(Campaign, c)
                    await asyncio.sleep(0)

                campaigns.extend(group_campaigns)

        return campaigns

    async def campaign_groups(self) -> List[CampaignGroup]:
        """
        Gets all campaign groups.
        """

        res = await self._get("/v1/campaigns/groups")

        campaign_groups: List[CampaignGroup] = []
        for cg in res:
            pydantic.tools.parse_obj_as(CampaignGroup, cg)
            await asyncio.sleep(0)

        return campaign_groups

    async def campaign_ideas(self, campaign_id: int) -> List[Idea]:
        """
        Gets all ideas from the campaign with the given id.
        """

        res = await self._get(f"/v1/campaigns/{campaign_id}/ideas")

        ideas = []
        for i in res:
            pydantic.tools.parse_obj_as(Idea, i)
            await asyncio.sleep(0)

        return ideas

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

                res_ideas: List[Idea] = []
                for i in res:
                    pydantic.tools.parse_obj_as(Idea, i)

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

        res = await self._get(f"/v1/funnels/{funnel_id}")
        return pydantic.tools.parse_obj_as(Funnel, res)

    async def _get(self, path: str) -> Mapping[str, Any] | Iterable[Mapping[str, Any]]:
        """
        Executes a GET request on IdeaScale API.
        """

        headers = {"api_token": self.api_token}
        return await self.inner.get(path, headers)
