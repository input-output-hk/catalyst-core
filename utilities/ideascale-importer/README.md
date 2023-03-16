IdeaScale Importer
===

## Getting Started

We recommend installing [pyenv](https://github.com/pyenv/pyenv#installation) to manage Python versions.

Install Python 3.11:

```sh
pyenv install 3.11
```

Install [Poetry](https://python-poetry.org/docs/#installation). Then install dependencies:

```sh
poetry env use python
poetry install
```

To see the available commands:

```sh
PYTHONPATH=$(pwd) poetry run python ideascale_importer --help
```

## Documentation

For documentation about the available commands see the [docs](docs) folder.

## Development

### Linting

```sh
# If you haven't already:
poetry run python -m flake8 ideascale_importer
```

### Type checking

```sh
poetry run python -m mypy ideascale_importer --check-untyped-defs
```
