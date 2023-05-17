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
  psql -e -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f ./setup/setup-db.sql \
    -v dbName=CatalystEventDev \
    -v dbDescription="Local Dev Catalayst Event DB" \
    -v dbUser="catalyst-event-dev"
  echo "Initializing database for graphql..."
  psql -e -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f ./setup/graphql-setup.sql \
    -v dbName=CatalystEventDev \
    -v dbUser="catalyst-event-dev" \
    -v adminUserFirstName="Admin" \
    -v adminUserLastName="Default" \
    -v adminUserAbout="Default Admin User" \
    -v adminUserEmail="admin.default@projectcatalyst.io"

  touch ./tmp/initialized.txt
else
  echo "Database already initialized. Skipping initialization."
fi

# Run migrations
./refinery migrate -e DATABASE_URL -c ./refinery.toml -p ./migrations

# Add historic data from previous funds
directory="./historic_data"

if [ -d "$directory" ];
then
for file in $(ls "$directory"/*.sql | sort); do
  echo "Adding historic data from $file"
  psql -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f "$file"
done
fi

# Add test data
directory="./test_data"

if [ -d "$directory" ];
then
for file in $(find "$directory" -name "*.sql" | sort); do
  echo "Adding test data from $file"
  psql -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f "$file"
done
fi
