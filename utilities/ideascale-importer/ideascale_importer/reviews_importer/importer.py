from lxml import html
import secrets
import time
from loguru import logger

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
        api_token,
        funnel_id,
        nr_allocations,
        stage_ids,
        pa_path,
        output_path,
    ):
        self.ideascale_url = ideascale_url
        self.database_url = database_url
        self.email = email
        self.password = password
        self.api_token = api_token
        self.funnel_id = funnel_id
        self.nr_allocations =  {i: el for i, el in enumerate(nr_allocations)}
        self.stage_ids = stage_ids

        self.pa_path = pa_path
        self.output_path = output_path

        self.frontend_client = None
        self.db = None

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
            nr_allocations=self.nr_allocations,
            pas_path=self.pa_path,
            ideascale_api_key=self.api_token,
            ideascale_api_url=self.ideascale_url,
            stage_ids=self.stage_ids,
            output_path=self.output_path,
        )


    async def run(self):
        """Run the importer."""
        if self.frontend_client is None:
            raise Exception("Not connected to the ideascale")

        # await self.download_reviews()
        await self.prepare_allocations()

    async def close(self):
        await self.frontend_client.close()

