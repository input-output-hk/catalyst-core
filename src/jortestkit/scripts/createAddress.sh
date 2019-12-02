#!/bin/sh

# Disclaimer:
#
#  The following use of shell script is for demonstration and understanding
#  only, it should *NOT* be used at scale or for any sort of serious
#  deployment, and is solely used for learning how the node and blockchain
#  works, and how to interact with everything.
#
#  It also asumes that `jcli` is in the same folder with the script.
#
#  Tutorials can be found here: https://iohk.zendesk.com/hc/en-us/categories/360002383814-Shelley-Networked-Testnet

### CONFIGURATION
CLI="./jcli"
ADDRTYPE="--testing"
PREFIX="--prefix addr"

if [ $# -ne 1 ]; then
    echo "usage: $0 <ADDR_TYPE>"
    echo "    <ADDR_TYPE>   Type of address to be created: account or utxo"
    exit 1
fi

ADDR_SK=$($CLI key generate --type=ed25519extended)
ADDR_PK=$(echo ${ADDR_SK} | $CLI key to-public)
if [ $1 = "account" ]; then
    ADDR=$($CLI address account ${PREFIX} ${ADDR_PK} ${ADDRTYPE})
elif [ $1 = "utxo" ]; then
    ADDR=$($CLI address single ${PREFIX} ${ADDR_PK} ${ADDRTYPE})
else
    echo "$1 - Unsupported value!"
    echo "Permitted values: account, utxo"
    exit 1
fi

echo "PRIVATE_KEY_SK: ${ADDR_SK}"
echo "PUBLIC_KEY_PK:  ${ADDR_PK}"
echo "ADDRESS:        ${ADDR}"
