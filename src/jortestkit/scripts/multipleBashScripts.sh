#!/bin/sh

### CONFIGURATION
CLI="jcli"
COLORS=1
ADDRTYPE="--testing"
TIMEOUT_NO_OF_BLOCKS=100
TX_VALUE=100
TOTAL_SOURCES=10
TOTAL_TRANSACTIONS_PER_SOURCE=10

if [ $# -ne 2 ]; then
  echo "usage: $0 <REST_LISTEN_PORT> <ACCOUNT_SK>"
  echo "    <REST_LISTEN_PORT>   The REST Listen Port set in node-config.yaml file (EX: 3101)"
  echo "    <ACCOUNT_SK>         The Secret key of the Source Account address (for transactions)"
  exit 1
fi

REST_PORT=$1
SOURCE_SK=$2

echo "#REST_PORT: ${REST_PORT}"
echo "#SOURCE_SK: ${SOURCE_SK}"


REST_URL="http://127.0.0.1:${REST_PORT}/api"

FEE_CONSTANT=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'constant:' | sed -e 's/^[[:space:]]*//' | sed -e 's/constant: //')
FEE_COEFFICIENT=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'coefficient:' | sed -e 's/^[[:space:]]*//' | sed -e 's/coefficient: //')
FEE_CERTIFICATE=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'certificate:' | sed -e 's/^[[:space:]]*//' | sed -e 's/certificate: //')
BLOCK0_HASH=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'block0Hash:' | sed -e 's/^[[:space:]]*//' | sed -e 's/block0Hash: //')
MAX_TXS_PER_BLOCK=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'maxTxsPerBlock:' | sed -e 's/^[[:space:]]*//' | sed -e 's/maxTxsPerBlock: //')
SLOT_DURATION=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'slotDuration:' | sed -e 's/^[[:space:]]*//' | sed -e 's/slotDuration: //')
SLOTS_PER_EPOCH=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'slotsPerEpoch:' | sed -e 's/^[[:space:]]*//' | sed -e 's/slotsPerEpoch: //')

### COLORS
if [ ${COLORS} -eq 1 ]; then
    GREEN=`printf "\033[0;32m"`
    RED=`printf "\033[0;31m"`
    BLUE=`printf "\033[0;33m"`
    WHITE=`printf "\033[0m"`
else
    GREEN=""
    RED=""
    BLUE=""
    WHITE=""
fi

### HELPERS

getTip() {
    echo $($CLI rest v0 tip get -h "${REST_URL}")
}

waitNewBlockCreated() {
  COUNTER=${TIMEOUT_NO_OF_BLOCKS}
  echo "  ##Waiting for new block to be created (timeout = $COUNTER blocks = $((${COUNTER}*${SLOT_DURATION}))s)"
  initialTip=$(getTip)
  actualTip=$(getTip)

  while [ "${actualTip}" = "${initialTip}" ]; do
    sleep ${SLOT_DURATION}
    actualTip=$(getTip)
    COUNTER=$((COUNTER - 1))
    if [ ${COUNTER} -lt 2 ]; then
      echo " !!!!! ERROR: Waited $(($COUNTER * $SLOT_DURATION))s secs ($COUNTER*$SLOT_DURATION) and no new block created"
      exit 1
    fi
  done
  echo "New block was created - $(getTip)"
}

###
#   1. create Account addresses for source
###

echo "Creating Source Account addresses"
for i in `seq 1 ${TOTAL_SOURCES}`;
do
    ACCOUNT_SK=$(jcli key generate --type=ed25519extended)
    ACCOUNT_PK=$(echo ${ACCOUNT_SK} | jcli key to-public)
    ACCOUNT_ADDR=$(jcli address account ${ACCOUNT_PK} --testing)

    list_of_source_sks[i]=${ACCOUNT_SK}
    list_of_source_addrs[i]=${ACCOUNT_ADDR}
done

##
#   2. send funds form Faucet to the above created Accounts
##
echo "Sending funds to Source Account addresses"
for i in `seq 1 ${#list_of_source_sks[@]}`;
do
    echo " == Sending funds to Account no: $i"
    bash send-money.sh ${list_of_source_addrs[$i]} 1000000000 ${REST_PORT} ${SOURCE_SK}
done

###
#   3. run multiple-transactions-per-slot.sh script providing source account from above as argument
###
START_TIME=`date +"%T"`;

bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[1]} | tee script1Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[2]} | tee script2Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[3]} | tee script3Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[4]} | tee script4Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[5]} | tee script5Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[6]} | tee script6Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[7]} | tee script7Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[8]} | tee script8Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[9]} | tee script9Logs.txt &
bash multiple-transactions-per-slot.sh ${TX_VALUE} ${REST_PORT} ${list_of_source_sks[10]} | tee script10Logs.txt &

wait
echo "all processes completed"
END_TIME=`date +"%T"`;

SEC1=`date +%s -d ${START_TIME}`
SEC2=`date +%s -d ${END_TIME}`

TX_TIME=$((${SEC2} - ${SEC1}))
echo "===== total test time for sending txs: ${TX_TIME} secs = `date +%H:%M:%S -ud @${TX_TIME}`"
echo "===== sent transactions: $((${TOTAL_SOURCES} * ${TOTAL_TRANSACTIONS_PER_SOURCE}))"
echo "===== TPS: $((1000/${TX_TIME}))"
