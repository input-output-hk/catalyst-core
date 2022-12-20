import marshmallow_dataclass
from typing import List, Mapping, Union


FieldMapping = Union[str, List[str]]


class ProposalsFieldsMappingConfig:
    proposer_url: FieldMapping
    proposer_relevant_experience: FieldMapping
    funds: FieldMapping
    public_key: FieldMapping


class ProposalsConfig:
    field_mappings: ProposalsFieldsMappingConfig
    extra_field_mappings: Mapping[str, FieldMapping]


class ProposalsScoresCsvConfig:
    id_field: str
    score_field: str


class Config:
    proposals: ProposalsConfig
    proposals_scores_csv: ProposalsScoresCsvConfig


ProposalsFieldsMappingConfigSchema = marshmallow_dataclass.class_schema(ProposalsFieldsMappingConfig)
ProposalsConfigSchema = marshmallow_dataclass.class_schema(ProposalsConfig)
ProposalsScoresCsvConfigSchema = marshmallow_dataclass.class_schema(ProposalsScoresCsvConfig)
ConfigSchema = marshmallow_dataclass.class_schema(Config)


def load(path: str) -> Config:
    with open(path) as f:
        config = ConfigSchema().loads(f.read())
        assert isinstance(config, Config)
        return config
