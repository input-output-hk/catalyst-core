#!/bin/sh

if [ -z "$DATABASE_URL" ]; then
  echo "ERROR: DATABASE_URL environment variable is not set."
  exit 1
fi

if [ -z "$JWT_SECRET" ]; then
  echo "ERROR: JWT_SECRET environment variable is not set."
  exit 1
fi

if [ -z "$GRAPHQL_PORT" ]; then
  GRAPHQL_PORT="5000"
  echo "WARNING: GRAPHQL_PORT environment variable is not set. Defaulting to 5000."
fi

npx -c ./cli.js \
    --connection "$DATABASE_URL" \
    --port "$GRAPHQL_PORT" \
    --cors \
    --subscriptions \
    --dynamic-json \
    --no-setof-functions-contain-nulls \
    --no-ignore-rbac \
    --show-error-stack=json \
    --extended-errors hint,detail,errcode \
    --append-plugins @graphile-contrib/pg-simplify-inflector \
    --export-schema-graphql schema.graphql \
    --graphiql "/" \
    --enhance-graphiql \
    --allow-explain \
    --enable-query-batching \
    --legacy-relations omit \
    --schema public,private \
    --default-role cat_anon \
    --jwt-secret "$JWT_SECRET" \
    --jwt-token-identifier private.jwt_token
