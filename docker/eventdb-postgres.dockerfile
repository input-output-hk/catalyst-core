FROM catalyst-core-base:latest

# Final image
FROM postgres:latest

# Scripts in `/docker-entrypoint-initdb.d` are executed in sorted name order as defined by the current locale (en_US.utf8).
# These file are executed only when `postgres_data` has not been initialized in the container
# This means that if there are failures when executing, the container storage needs to be cleared before retrying. See docs in `https://hub.docker.com/_/postgres/` for further information.

COPY --from=0 /usr/src/catalyst-core/src/event-db/setup/setup-db.sql /docker-entrypoint-initdb.d/db.sql
COPY --from=0 /usr/src/catalyst-core/src/event-db/setup/graphql-setup.sql /docker-entrypoint-initdb.d/graphql.sql
