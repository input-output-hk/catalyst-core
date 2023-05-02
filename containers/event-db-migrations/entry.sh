#!/usr/bin/env sh

# Set default value for reinitDb flag
reinitDb=false

# Check if reinitDb flag is set in environment variable
if [ -n "$REINIT_DB" ]; then
  if [ "$REINIT_DB" = true ] || [ "$REINIT_DB" = false ]; then
    reinitDb=$REINIT_DB
  else
    echo "Invalid value for REINIT_DB environment variable. Must be true or false."
    exit 1
  fi
fi

# Check if the database has been initialized
if [ ! -f ./tmp/initialized.txt ] || [ "$reinitDb" = true ]; then
  echo "Initializing database..."
  psql -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f ./setup/setup-db.sql
  echo "Initializing database for graphql..."
  psql -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f ./setup/graphql-setup.sql
  touch ./tmp/initialized.txt
else
  echo "Database already initialized. Skipping initialization."
fi

# Run migrations
./refinery migrate -e DATABASE_URL -c ./refinery.toml -p ./migrations

# Add historic data from previous funds
directory="./historic_data"

for file in $(ls "$directory"/*.sql | sort); do
  echo "Adding historic data from $file"
  psql -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f "$file"
done
