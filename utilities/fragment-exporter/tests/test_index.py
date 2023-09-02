import os
import tempfile
from fragment_exporter.index import Index

import pytest


@pytest.fixture
def temporary_db():
    # Setup: create a temporary directory
    temp_dir = tempfile.TemporaryDirectory()
    yield temp_dir.name
    # Teardown: close the directory after the test ends
    temp_dir.cleanup()


@pytest.fixture
def index(temporary_db):
    return Index(temporary_db)


def test_init_db(index):
    assert os.path.exists(index.db_path)


def test_insert_and_exists(index):
    fragment_id = "test_fragment"
    assert not index.exists(fragment_id)
    index.insert(fragment_id)
    assert index.exists(fragment_id)


def test_set_and_get_last_block_height(index):
    height = 1234
    assert index.get_last_block_height() == 0
    index.set_last_block_height(height)
    assert index.get_last_block_height() == height


def test_purge(index):
    fragment_id = "test_fragment"
    index.insert(fragment_id)
    assert index.exists(fragment_id)
    index.purge()
    assert not index.exists(fragment_id)
