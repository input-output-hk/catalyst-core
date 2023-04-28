#!/usr/bin/env sh

# Set default value for initDb flag
initDb=false

# Check if initDb flag is set in environment variable
if [ -n "$INIT_DB" ]; then
  if [ "$INIT_DB" = true ] || [ "$INIT_DB" = false ]; then
    initDb=$INIT_DB
  else
    echo "Invalid value for INIT_DB environment variable. Must be true or false."
    exit 1
  fi
fi

# Parse command line arguments
for arg in "$@"; do
  case $arg in
    -i|--initDb)
      initDb=true
      ;;
    *)
      # unknown option
      echo "Unknown option: $arg"
      exit 1
      ;;
  esac
done

# Check if initDb flag is set
if [ "$initDb" = true ]; then
  echo "Initializing database..."
  psql -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f /setup/setup-db.sql
  echo "Initializing database for graphql..."
  psql -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f /setup/graphql-setup.sql
else
  echo "initDb flag not set. Skipping database initialization."
fi

# Run migrations
./refinery migrate -e DATABASE_URL -c ./refinery.toml -p /migrations

# Add historic data from previous funds
directory="/historic_data"

for file in $(ls "$directory"/*.sql | sort); do
  echo "Adding historic data from $file"
  psql -U $PGUSER -d $PGDATABASE -h $PGHOST -p $PGPORT -f "$file"
done
