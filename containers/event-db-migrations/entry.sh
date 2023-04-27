#!/usr/bin/env sh
./refinery migrate -e DATABASE_URL -c ./refinery.toml -p /migrations
psql -V #-U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT
