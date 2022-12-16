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


class Config:
    proposals: ProposalsConfig


ProposalsFieldsMappingConfigSchema = marshmallow_dataclass.class_schema(ProposalsFieldsMappingConfig)
ProposalsConfigSchema = marshmallow_dataclass.class_schema(ProposalsConfig)
ConfigSchema = marshmallow_dataclass.class_schema(Config)


def load(path: str) -> Config:
    with open(path) as f:
        config = ConfigSchema().loads(f.read())
        assert isinstance(config, Config)
        return config
