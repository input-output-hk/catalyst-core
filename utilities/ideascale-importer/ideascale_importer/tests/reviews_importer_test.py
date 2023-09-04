import pytest
import os

from ideascale_importer.reviews_importer.importer import Importer

def get_env_var(env_var):
    if os.environ.get(env_var):
        return os.environ.get(env_var)
    else:
        raise Exception(f"could not find {env_var} env var")

@pytest.fixture(scope="session")
def reviews_importer_mock(tmpdir_factory):
    ideascale_url = get_env_var("IDEASCALE_API_URL")
    database_url = get_env_var("EVENTDB_URL")
    email = get_env_var("IDEASCALE_EMAIL")
    password = get_env_var("IDEASCALE_PASSWORD")
    event_id = 0
    api_token = get_env_var("IDEASCALE_API_TOKEN")
    allocations_path = "./ideascale_importer/tests/test_data/allocations.csv"
    output_path = tmpdir_factory.mktemp("output")
    return Importer(
            ideascale_url=ideascale_url,
            database_url=database_url,
            email=email,
            password=password,
            api_token=api_token,
            event_id=event_id,
            allocations_path=allocations_path,
            output_path=output_path
        )

@pytest.mark.asyncio
async def test_reviews_importer(reviews_importer_mock):
    await reviews_importer_mock.connect()
    await reviews_importer_mock.run()
    await reviews_importer_mock.close()