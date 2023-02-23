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
poetry run python src/main.py --help
```

## Importing IdeaScale Data

The easiest way is to run:

```sh
poetry run python src/main.py \
    --api-token IDEASCALE_API_TOKEN \
    --database-url POSTGRES_URL
```

And go through the interactive steps.

## Development

### Linting

```sh
# If you haven't already:
poetry run python -m flake8 src
```

### Type checking

```sh
poetry run python -m mypy src --check-untyped-defs
```
