# Importing IdeaScale Data

The easiest way is to run:

```sh
PYTHONPATH=$(pwd) poetry run python ideascale_importer \
    --api-token IDEASCALE_API_TOKEN \
    --database-url POSTGRES_URL
```

And go through the interactive steps.
