IdeaScale Importer
===

## Getting Started

> *Python 3 is required*

To install all the dependencies without polluting your Python installation run:

```sh
pip install -r requirements.txt
```

To see the available commands:

```sh
python src/main.py --help
```

## Importing IdeaScale Data

The easiest way is to run:

```sh
python src/main.py postgres import-all \
    --api-token IDEASCALE_API_TOKEN \
    --database-url POSTGRES_URL
```

And go through the interactive steps.

## Development

### Linting

```sh
# If you haven't already:
python -m flake8 src
```

### Type checking

```sh
python -m mypy src --check-untyped-defs
```
