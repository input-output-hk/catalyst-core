import dataclasses
import typing

from .models import Model


def insert_statement(model: Model) -> str:
    field_names = []
    field_values = []

    for field in dataclasses.fields(model):
        field_value = getattr(model, field.name)
        if field_value is not None:
            if field.type is str or \
               typing.get_origin(field.type) is typing.Union and typing.get_args(field.type) == (str, type(None)):
                field_values.append(f"'{field_value}'")
            else:
                field_values.append(str(field_value))
            field_names.append(field.name)

    field_names_str = ",".join(field_names)
    field_values_str = ",".join(field_values)

    return f"INSERT INTO {model.table()} ({field_names_str}) VALUES ({field_values_str})"
