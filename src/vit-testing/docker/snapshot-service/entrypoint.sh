#!/bin/sh
{
  echo "{"
  echo "	\"address\": \"0.0.0.0:8080\","
  echo "	\"result-dir\": \"./data\","
  echo "	\"voting-tools\": {"
} > /config.yaml

if [ $TESTNET_MAGIC ]; then
    {
      echo "		\"network\": {"
      echo "			\"testnet\": $TESTNET_MAGIC"
      echo "		},"
    } >> /config.yaml
else
    echo "		\"network\": \"mainnet\"," >> /config.yaml
fi
{
  echo "		\"bin\": \"/root/.cargo/bin/snapshot_tool\","
  echo "		\"db\":  \"$DB_NAME\","
  echo "		\"db-user\":  \"$DB_USER\","
  echo "		\"db-pass\":  \"$DB_PASS\","
  echo "		\"db-host\":  \"$DB_HOST\""
  echo "	}"
  echo "}"
} >> /config.yaml
exec "$@"