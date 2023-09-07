from lxml import html
import time
from loguru import logger
from typing import List, Dict
import pydantic
import tempfile
import json

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

class ReviewsManager:
    def __init__(
        self,
        ideascale_url,
        api_token,
        config_path = None,
        email = None,
        password = None,
    ):
        self.ideascale_url = ideascale_url
        self.email = email
        self.password = password
        self.api_token = api_token
        self.config_path = config_path

        self.frontend_client = None
        self.db = None

    async def connect(self):
        if self.frontend_client is None and (self.email and self.password):
            logger.info("Connecting to the Ideascale frontend")
            self.frontend_client = FrontendClient(self.ideascale_url)
            await self.frontend_client.login(self.email, self.password)

    async def __load_config(self):
        """Load the configuration setting from the event db."""

        logger.info(f"Loading ideascale config from {self.config_path}")
        config_data = json.load(open(self.config_path))
        self.config = Config(**config_data)

    async def __download_reviews(self, reviews_output_path: str):
        logger.info("Download reviews from Ideascale...")

        self.reviews = await self.frontend_client.download_reviews(reviews_output_path, self.config.review_stage_ids)

    # This code will be moved as a separate cli command
    async def __prepare_allocations(self, pas_path: str, output_path: str):
        logger.info("Prepare allocations for proposal's reviews...")

        await allocate(
            nr_allocations=self.config.nr_allocations,
            pas_path=pas_path,
            ideascale_api_key=self.api_token,
            ideascale_api_url=self.ideascale_url,
            stage_ids=self.config.stage_ids,
            challenges_group_id=self.config.campaign_group_id,
            group_id=self.config.group_id,
            anonymize_start_id=self.config.anonymize_start_id,
            output_path=output_path,
        )
    
    async def __prepare_reviews(self, allocations_path: str, output_path: str):
        logger.info("Prepare proposal's reviews...")
        await process_ideascale_reviews(
            ideascale_xlsx_path=self.reviews,
            ideascale_api_url=self.ideascale_url,
            ideascale_api_key=self.api_token,
            allocation_path=allocations_path,
            challenges_group_id=self.config.campaign_group_id,
            questions=self.config.questions,
            fund=self.config.event_id,
            output_path=output_path
        )

    async def __import_reviews_to_service(self):
        logger.info("Import reviews into cat data service")

    async def generate_allocations_run(self, pas_path: str, output_path: str):
        """Run the Allocations generation."""

        await self.__load_config()
        await self.__prepare_allocations(pas_path=pas_path, output_path=output_path)

    async def import_reviews_run(self, allocations_path: str, output_path: str):
        """Run the reviews importer."""
        if self.frontend_client is None:
            raise Exception("Not connected to the ideascale")

        await self.__load_config()
        reviews_dir = tempfile.TemporaryDirectory()
        await self.__download_reviews(reviews_output_path=reviews_dir.name)
        await self.__prepare_reviews(allocations_path=allocations_path, output_path=output_path)

        reviews_dir.cleanup()

    async def close(self):
        if self.frontend_client:
            await self.frontend_client.close()

class Config(pydantic.BaseModel):
    """Represents the available configuration fields."""

    event_id: int
    group_id: int
    campaign_group_id: int
    review_stage_ids: List[int]
    stage_ids: List[int]
    nr_allocations: List[int]
    questions: Dict[str, str]
    anonymize_start_id: int
