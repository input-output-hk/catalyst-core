from lxml import html
import time
from loguru import logger
from dataclasses import dataclass
from typing import List, Dict
import pydantic
import tempfile

from ideascale_importer import utils
import ideascale_importer.db
from .processing.prepare import allocate, process_ideascale_reviews


class FrontendClient:
    """IdeaScale front-end client."""

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

    async def download_reviews(self, reviews_path, review_stage_ids):
        async def download_file(self, review_stage_id):
            export_endpoint = "/a/admin/workflow/survey-tools/assessment/report/statistic/export/assessment-details/"
            file_name = f"{reviews_path}/{review_stage_id}.xlsx"

            content = await self.inner.get(f"{export_endpoint}{review_stage_id}") 
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
                    f = open(file_name, "wb")
                    f.write(content)
                    return file_name

        files = []
        for review_stage_id in review_stage_ids:
            # we are interested in only assessed reviews 
            files.append(await download_file(self, review_stage_id))
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

        self.reviews_dir = tempfile.TemporaryDirectory()
        self.allocations_dir = tempfile.TemporaryDirectory()

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

        self.reviews = await self.frontend_client.download_reviews(self.reviews_dir.name, self.config.review_stage_ids)

    async def prepare_allocations(self):
        logger.info("Prepare allocations for proposal's reviews...")

        self.allocations_path = await allocate(
            nr_allocations=self.config.nr_allocations,
            pas_path=self.pa_path,
            ideascale_api_key=self.api_token,
            ideascale_api_url=self.ideascale_url,
            stage_ids=self.config.stage_ids,
            challenges_group_id=self.config.campaign_group_id,
            group_id=self.config.group_id,
            output_path=self.allocations_dir.name,
        )
    
    async def prepare_reviews(self):
        logger.info("Prepare proposal's reviews...")
        await process_ideascale_reviews(
            ideascale_xlsx_path=self.reviews,
            ideascale_api_url=self.ideascale_url,
            ideascale_api_key=self.api_token,
            allocation_path=self.allocations_path,
            challenges_group_id=self.config.campaign_group_id,
            questions=self.config.questions,
            fund=self.event_id,
            output_path=self.output_path
        )

    async def import_reviews(self):
        logger.info("Import reviews into Event db")

    async def run(self):
        """Run the importer."""
        if self.frontend_client is None:
            raise Exception("Not connected to the ideascale")

        await self.load_config()

        await self.download_reviews()
        await self.prepare_allocations()
        await self.prepare_reviews()

    async def close(self):
        self.reviews_dir.cleanup()
        self.allocations_dir.cleanup()
        await self.frontend_client.close()

@dataclass
class Config:
    """Represents the available configuration fields."""

    group_id: int
    campaign_group_id: int
    review_stage_ids: List[int]
    stage_ids: List[int]
    nr_allocations: List[int]
    questions: Dict[str, str]
    
    @staticmethod
    def from_json(val: dict):
        """Load configuration from a JSON object."""
        return pydantic.tools.parse_obj_as(Config, val)

