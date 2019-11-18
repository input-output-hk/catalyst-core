#!/bin/sh

### CONFIGURATION
CLI="jcli"
COLORS=1
ADDRTYPE="--testing"

if [ $# -ne 1 ]; then
    echo "usage: $0 <ACCOUNT_SK>"
    echo "    <ACCOUNT_SK>   The Secret key of the Source address"
    exit 1
fi

ACCOUNT_SK="$1"

ACCOUNT_PK=$(echo ${ACCOUNT_SK} | $CLI key to-public)
ACCOUNT_ADDR=$($CLI address account ${ADDRTYPE} ${ACCOUNT_PK})

echo "================Create and sign Stake Pool certificate ================="
echo "ACCOUNT_SK: ${ACCOUNT_SK}"
echo "=================================================="

echo " ##1. Create VRF keys"
POOL_VRF_SK=$($CLI key generate --type=Curve25519_2HashDH)
POOL_VRF_PK=$(echo ${POOL_VRF_SK} | $CLI key to-public)

echo POOL_VRF_SK: ${POOL_VRF_SK}
echo POOL_VRF_PK: ${POOL_VRF_PK}

echo " ##2. Create KES keys"
POOL_KES_SK=$($CLI key generate --type=SumEd25519_12)
POOL_KES_PK=$(echo ${POOL_KES_SK} | $CLI key to-public)

echo POOL_KES_SK: ${POOL_KES_SK}
echo POOL_KES_PK: ${POOL_KES_PK}

echo " ##3. Create the Stake Pool certificate using above VRF and KES public keys"
ACCOUNT_SK_FILE="account.privateKey"
STAKE_POOL_CERTIFICATE_FILE="stake_pool.cert"
SIGNED_STAKE_POOL_CERTIFICATE_FILE="stake_pool_certificate.signed"

echo ${ACCOUNT_SK} > ${ACCOUNT_SK_FILE}

$CLI certificate new stake-pool-registration --kes-key ${POOL_KES_PK} --vrf-key ${POOL_VRF_PK} --owner ${ACCOUNT_PK} --serial 1010101010 --start-validity 0 --management-threshold 1 ${STAKE_POOL_CERTIFICATE_FILE}

echo " ##4. Sign the Stake Pool certificate with the Stake Pool Owner private key"

$CLI certificate sign \
    --certificate ${STAKE_POOL_CERTIFICATE_FILE} \
    --key ${ACCOUNT_SK_FILE} \
    --output ${SIGNED_STAKE_POOL_CERTIFICATE_FILE}

$CLI certificate sign \
    --certificate ${STAKE_POOL_CERTIFICATE_FILE} \
    --key ${ACCOUNT_SK_FILE} \
    --output ${SIGNED_STAKE_POOL_CERTIFICATE_FILE}

echo "SIGNED_STAKE_POOL_CERTIFICATE: $(cat ${SIGNED_STAKE_POOL_CERTIFICATE_FILE})"

echo " ##5. Retrieve your stake pool id (NodeId)"
NODE_ID=$($CLI certificate get-stake-pool-id ${SIGNED_STAKE_POOL_CERTIFICATE_FILE})

echo "The Node ID is: ${NODE_ID}"

rm ${ACCOUNT_SK_FILE} ${STAKE_POOL_CERTIFICATE_FILE} ${SIGNED_STAKE_POOL_CERTIFICATE_FILE}

echo " ##6. Create the node_secret.yaml file"
#define the template for node_secret.yaml file
cat > node_secret.yaml << EOF
genesis:
  sig_key: ${POOL_KES_SK}
  vrf_key: ${POOL_VRF_SK}
  node_id: ${NODE_ID}
EOF