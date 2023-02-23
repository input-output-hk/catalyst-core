import marshmallow_dataclass
from typing import List, Mapping, Union


FieldMapping = Union[str, List[str]]


class ProposalsFieldsMappingConfig:
    """
    Represents the available configuration fields used in proposal fields mapping.
    """

    proposer_url: FieldMapping
    proposer_relevant_experience: FieldMapping
    funds: FieldMapping
    public_key: FieldMapping


class ProposalsConfig:
    """
    Represents the available configuration fields used in proposal processing.
    """

    field_mappings: ProposalsFieldsMappingConfig
    extra_field_mappings: Mapping[str, FieldMapping]


class ProposalsScoresCsvConfig:
    """
    Represents the available configuration fields used
    when loading proposal scores from the CSV file.
    """

    id_field: str
    score_field: str


class Config:
    """
    Represents the available configuration fields.
    """

    proposals: ProposalsConfig
    proposals_scores_csv: ProposalsScoresCsvConfig


ProposalsFieldsMappingConfigSchema = marshmallow_dataclass.class_schema(ProposalsFieldsMappingConfig)
ProposalsConfigSchema = marshmallow_dataclass.class_schema(ProposalsConfig)
ProposalsScoresCsvConfigSchema = marshmallow_dataclass.class_schema(ProposalsScoresCsvConfig)
ConfigSchema = marshmallow_dataclass.class_schema(Config)


def from_json_file(path: str) -> Config:
    """
    Loads configuration from a JSON file.
    """

    with open(path) as f:
        config = ConfigSchema().loads(f.read())
        assert isinstance(config, Config)
        return config
