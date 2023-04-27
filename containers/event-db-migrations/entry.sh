#!/usr/bin/env sh
./refinery migrate -c ./refinery.toml -p /migrations
psql -V
