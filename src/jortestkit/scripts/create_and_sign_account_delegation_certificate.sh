#!/bin/sh

### CONFIGURATION
CLI="./jcli"
COLORS=1
ADDRTYPE="--testing"

if [ $# -ne 2 ]; then
    echo "usage: $0 <STAKE_POOL_ID> <ACCOUNT_SK>"
    echo "    <STAKE_POOL_ID>  The ID of the Stake Pool you want to delegate to"
    echo "    <ACCOUNT_SK>   The Secret key of the Source address"
    exit 1
fi

STAKE_POOL_ID="$1"
ACCOUNT_SK="$2"

ACCOUNT_PK=$(echo ${ACCOUNT_SK} | $CLI key to-public)
ACCOUNT_ADDR=$($CLI address account ${ADDRTYPE} ${ACCOUNT_PK})

echo "================Create and sign Account certificate ================="
echo "ACCOUNT_SK: ${ACCOUNT_SK}"
echo "=================================================="

echo " ##1. Create the delegation certificate for the Account address"

ACCOUNT_SK_FILE="account.prv"
CERTIFICATE_FILE="account_delegation_certificate"
SIGNED_CERTIFICATE_FILE="account_delegation_certificate.signed"
echo ${ACCOUNT_SK} > ${ACCOUNT_SK_FILE}

$CLI certificate new stake-delegation \
    ${ACCOUNT_PK} \
    ${STAKE_POOL_ID} \
    --output ${CERTIFICATE_FILE}

echo " ##2. Sign the delegation certificate for the Account address"

$CLI certificate sign \
    --certificate ${CERTIFICATE_FILE} \
    --key ${ACCOUNT_SK_FILE} \
    --output ${SIGNED_CERTIFICATE_FILE}

echo "SIGNED_CERTIFICATE_FILE: $(cat ${SIGNED_CERTIFICATE_FILE})"

rm ${ACCOUNT_SK_FILE}
