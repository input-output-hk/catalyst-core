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
```

Initialize poetry's virtual environment.

```sh
poetry shell
```

Install the package.

```sh
poetry install
```

To see the available commands:

```sh
poetry run python -m ideascale_importer.app --help
```

or

```sh
python -m ideascale_importer --help
```

or, use the executable script:

```sh
ideascale-importer --help
```

To leave the virtual environment, just type:

```sh
exit
```

## Documentation

For documentation about the available commands see the [docs](docs) folder.

## Development

### Formatting

```sh
poetry run black ideascale_importer
```

### Linting

```sh
# If you haven't already:
poetry run ruff check ideascale_importer
```

### Type checking

```sh
poetry run mypy ideascale_importer --check-untyped-defs
```
