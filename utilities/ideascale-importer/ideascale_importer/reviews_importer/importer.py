from lxml import html
import secrets
import time
from loguru import logger
from dataclasses import dataclass
from typing import List
import pydantic

from ideascale_importer import utils
import ideascale_importer.db
from .processing.prepare import allocate


class FrontendClient:
    """IdeaScale front-end client."""

    DEFAULT_API_URL = "https://cardano.ideascale.com"

    def __init__(self, ideascale_url):
        self.inner = utils.HttpClient(ideascale_url)

    async def close(self):
        await self.inner.close()

    async def login(self,  email, password):
        login = "/a/community/login"

        data = {
            'target-content-type': 'json',
            'emailAddress': email,
            'password': password,
            'rememberMe': 'true',
        }

        await self.inner.post(f"{login}", data=data)

    async def download_reviews(self, funnel_id):
        async def download_file(self, funnel_id, id):
            export_endpoint = "/a/admin/workflow/survey-tools/assessment/report/statistic/export/assessment-details/"
            file_name = f"{funnel_id}_{secrets.token_hex(6)}"

            content = await self.inner.get(f"{export_endpoint}{id}") 
            tree = html.fromstring(content)

            # we are looking for '<div class="card panel export-result-progress" data-features="refresh-processing-item" data-processing-item-id="15622">'
            # to get "15622" value as for example
            item  = int(tree.find(".//div[@data-processing-item-id]").get("data-processing-item-id"))

            export_data_endpoint = "/a/reporting/export-data/"
            while True:
                time.sleep(2)
                content = await self.inner.get(f"{export_data_endpoint}{item}")
                if "Finished Processing" in str(content):
                    download_endpoint = "/a/download-export-file/"

                    content = await self.inner.get(f"{download_endpoint}{item}")
                    return content


        funnel_endpoint = "/a/admin/workflow/stages/funnel/"
        content = await self.inner.get(f"{funnel_endpoint}{funnel_id}")

        # we are looking for '<a href="/a/admin/workflow/survey-tools/assessment/report/statistic/139?fromStage=1">Assessments</a>'
        # where we need to get url
        tree = html.fromstring(content)
        items = tree.findall('.//a')
        files = []
        for item in items:
            if item.text and "Assessments" in item.text:
                id = int(item.get("href").replace("/a/admin/workflow/survey-tools/assessment/report/reviews/", "").split("?")[0])

                # we are intrested in only assessed reviews 
                files.append(await download_file(self, funnel_id, id))
        return files

class Importer:
    def __init__(
        self,
        ideascale_url,
        database_url,
        email,
        password,
        event_id,
        api_token,
        pa_path,
        output_path,
    ):
        self.ideascale_url = ideascale_url
        self.database_url = database_url
        self.email = email
        self.password = password
        self.event_id = event_id
        self.api_token = api_token

        self.pa_path = pa_path
        self.output_path = output_path

        self.frontend_client = None
        self.db = None

    async def load_config(self):
        """Load the configuration setting from the event db."""

        logger.info("Loading ideascale config from the event-db")

        config = ideascale_importer.db.models.Config(row_id=0, id="ideascale", id2=f"{self.event_id}", id3="", value=None)
        res = await ideascale_importer.db.select(self.db, config,  cond={
            "id": f"= '{config.id}'",
            "AND id2": f"= '{config.id2}'"
            })
        if len(res) == 0:
            raise Exception("Cannot find ideascale config in the event-db database")
        self.config = Config.from_json(res[0].value)

    async def connect(self):
        if self.frontend_client is None:
            logger.info("Connecting to the Ideascale frontend")
            self.frontend_client = FrontendClient(self.ideascale_url)
            await self.frontend_client.login(self.email, self.password)
        if self.db is None:
            logger.info("Connecting to the database")
            self.db = await ideascale_importer.db.connect(self.database_url)

    async def download_reviews(self):
        logger.info("Dowload reviews from Ideascale...")
        await self.frontend_client.download_reviews(self.funnel_id)

    # This step does not using right now, need to getting PAs files and uploading some results to Ideascale
    async def prepare_allocations(self):
        logger.info("Prepare allocations for proposal's reviews...")

        await allocate(
            nr_allocations=self.config.nr_allocations,
            pas_path=self.pa_path,
            ideascale_api_key=self.api_token,
            ideascale_api_url=self.ideascale_url,
            stage_ids=self.config.stage_ids,
            challenges_group_id=self.config.campaign_group_id,
            group_id=self.config.group_id,
        )


    async def run(self):
        """Run the importer."""
        if self.frontend_client is None:
            raise Exception("Not connected to the ideascale")

        await self.load_config()

        # await self.download_reviews()
        await self.prepare_allocations()

    async def close(self):
        await self.frontend_client.close()

@dataclass
class Config:
    """Represents the available configuration fields."""

    group_id: int
    campaign_group_id: int
    funnel_id: int
    stage_ids: List[int]
    nr_allocations: List[int]
    
    @staticmethod
    def from_json(val: dict):
        """Load configuration from a JSON object."""
        return pydantic.tools.parse_obj_as(Config, val)

