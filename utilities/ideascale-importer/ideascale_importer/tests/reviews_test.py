import pytest
import os

from ideascale_importer.reviews_manager.manager import ReviewsManager

def get_env_var(env_var):
    if os.environ.get(env_var):
        return os.environ.get(env_var)
    else:
        raise Exception(f"could not find {env_var} env var")

@pytest.fixture()
def reviews_manager_mock():
    ideascale_url = get_env_var("IDEASCALE_API_URL")
    email = get_env_var("IDEASCALE_EMAIL")
    password = get_env_var("IDEASCALE_PASSWORD")
    api_token = get_env_var("IDEASCALE_API_TOKEN")
    config_path = "./ideascale_importer/tests/test_data/config.json"
    return ReviewsManager(
            ideascale_url=ideascale_url,
            email=email,
            password=password,
            api_token=api_token,
            config_path=config_path
        )

@pytest.mark.asyncio
async def test_reviews_importer(reviews_manager_mock, tmpdir_factory):
    allocations_path = "./ideascale_importer/tests/test_data/allocations.csv"
    output_path = tmpdir_factory.mktemp("output")

    await reviews_manager_mock.connect()
    await reviews_manager_mock.import_reviews_run(
            allocations_path=allocations_path,
            output_path=output_path
        )
    await reviews_manager_mock.close()

@pytest.mark.asyncio
async def test_allocations_generator(reviews_manager_mock, tmpdir_factory):
    pas_path = "./ideascale_importer/tests/test_data/pas.csv"
    output_path = tmpdir_factory.mktemp("output")

    await reviews_manager_mock.connect()
    await reviews_manager_mock.generate_allocations_run(
            pas_path=pas_path,
            output_path=output_path
        )
    await reviews_manager_mock.close()