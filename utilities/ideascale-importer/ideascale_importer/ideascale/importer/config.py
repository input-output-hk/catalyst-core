from pydantic.dataclasses import dataclass
import json
import pydantic.tools
from typing import List, Mapping, Union


FieldMapping = Union[str, List[str]]


@dataclass
class ProposalsFieldsMappingConfig:
    """
    Represents the available configuration fields used in proposal fields mapping.
    """

    proposer_url: FieldMapping
    proposer_relevant_experience: FieldMapping
    funds: FieldMapping
    public_key: FieldMapping


@dataclass
class ProposalsConfig:
    """
    Represents the available configuration fields used in proposal processing.
    """

    field_mappings: ProposalsFieldsMappingConfig
    extra_field_mappings: Mapping[str, FieldMapping]


@dataclass
class ProposalsScoresCsvConfig:
    """
    Represents the available configuration fields used
    when loading proposal scores from the CSV file.
    """

    id_field: str
    score_field: str


@dataclass
class Config:
    """
    Represents the available configuration fields.
    """

    proposals: ProposalsConfig
    proposals_scores_csv: ProposalsScoresCsvConfig


def from_json_file(path: str) -> Config:
    """
    Loads configuration from a JSON file.
    """

    with open(path) as f:
        return pydantic.tools.parse_obj_as(Config, json.load(f))

