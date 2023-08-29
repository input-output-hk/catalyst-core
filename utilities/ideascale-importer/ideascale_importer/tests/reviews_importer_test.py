import pytest
import tempfile

from ideascale_importer.reviews_importer.importer import Importer

@pytest.fixture
def reviews_importer_mock():
    ideascale_url = "https://temp-cardano-sandbox.ideascale.com"
    database_url = "postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"
    email = ""
    password = ""
    event_id = 0
    api_token = ""
    pa_path = "./pa.csv"
    output_path = tempfile.TemporaryDirectory()
    return Importer(
            ideascale_url=ideascale_url,
            database_url=database_url,
            email=email,
            password=password,
            api_token=api_token,
            event_id=event_id,
            pa_path=pa_path,
            output_path=output_path.name
        )

@pytest.mark.asyncio
async def test_reviews_importer(reviews_importer_mock):
    await reviews_importer_mock.connect()
    await reviews_importer_mock.run()
    await reviews_importer_mock.close()