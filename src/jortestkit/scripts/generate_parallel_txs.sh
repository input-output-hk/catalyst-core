#!/bin/sh

### CONFIGURATION
CLI="jcli"
COLORS=1
ADDRTYPE="--testing"
TIMEOUT_NO_OF_BLOCKS=100
INIT_TX_VALUE=1000000
TOTAL_SOURCES=10
TOTAL_TRANSACTIONS_PER_SOURCE=500

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

getNoOfMinedTransactions() {
    echo $($CLI rest v0 message logs -h "${REST_URL}" | tr ' ' '\n' | grep 'InABlock:' | wc -l)
}

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

sendMoney() {
    DST_ADDR="$1"

    # Account 1 pays for the transaction fee
    TX_AMOUNT=$((${INIT_TX_VALUE} + ${FEE_CONSTANT} + $((2 * ${FEE_COEFFICIENT}))))
    STAGING_FILE="acc_staging.$$.transaction"

    # Create the transaction
    $CLI transaction new --staging ${STAGING_FILE}
    $CLI transaction add-account "${SRC_ADDR}" "${TX_AMOUNT}" --staging "${STAGING_FILE}"
    $CLI transaction add-output "${DST_ADDR}" "${INIT_TX_VALUE}" --staging "${STAGING_FILE}"
    $CLI transaction finalize --staging ${STAGING_FILE}

    TRANSACTION_DATA=$($CLI transaction data-for-witness --staging ${STAGING_FILE})

    # Create the witness for the 1 input (add-account) and add it
    SRC_WITNESS_SECRET_FILE="witness.secret.$$"
    SRC_WITNESS_OUTPUT_FILE="witness.out.$$"

    printf "${SOURCE_SK}" > ${SRC_WITNESS_SECRET_FILE}

    $CLI transaction make-witness ${TRANSACTION_DATA} \
        --genesis-block-hash ${BLOCK0_HASH} \
        --type "account" --account-spending-counter "${SRC_COUNTER_INCR}" \
        ${SRC_WITNESS_OUTPUT_FILE} ${SRC_WITNESS_SECRET_FILE}
    $CLI transaction add-witness ${SRC_WITNESS_OUTPUT_FILE} --staging "${STAGING_FILE}"

    # Finalize the transaction and send it
    $CLI transaction seal --staging "${STAGING_FILE}"
    tx_hash=$($CLI transaction to-message --staging "${STAGING_FILE}" | $CLI rest v0 message post -h "${REST_URL}")

    rm -f ${STAGING_FILE} ${SRC_WITNESS_SECRET_FILE} ${SRC_WITNESS_OUTPUT_FILE}
}

if [ $# -ne 2 ]; then
  echo "usage: $0 <REST_LISTEN_PORT> <ACCOUNT_SK>"
  echo "    <REST_LISTEN_PORT>   The REST Listen Port set in node-config.yaml file (EX: 3101)"
  echo "    <ACCOUNT_SK>         The Secret key of the Source Account address (for transactions)"
  exit 1
fi

REST_PORT=$1
SOURCE_SK=$2

REST_URL="http://127.0.0.1:${REST_PORT}/api"

SOURCE_PK=$(echo ${SOURCE_SK} | $CLI key to-public)
SRC_ADDR=$($CLI address account ${ADDRTYPE} ${SOURCE_PK})
SOURCE_COUNTER=$( $CLI rest v0 account get "${SRC_ADDR}" -h "${REST_URL}" | grep '^counter:' | sed -e 's/counter: //' )

FEE_CONSTANT=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'constant:' | sed -e 's/^[[:space:]]*//' | sed -e 's/constant: //')
FEE_COEFFICIENT=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'coefficient:' | sed -e 's/^[[:space:]]*//' | sed -e 's/coefficient: //')
FEE_CERTIFICATE=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'certificate:' | sed -e 's/^[[:space:]]*//' | sed -e 's/certificate: //')
BLOCK0_HASH=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'block0Hash:' | sed -e 's/^[[:space:]]*//' | sed -e 's/block0Hash: //')
MAX_TXS_PER_BLOCK=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'maxTxsPerBlock:' | sed -e 's/^[[:space:]]*//' | sed -e 's/maxTxsPerBlock: //')
SLOT_DURATION=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'slotDuration:' | sed -e 's/^[[:space:]]*//' | sed -e 's/slotDuration: //')
SLOTS_PER_EPOCH=$($CLI rest v0 settings get -h "${REST_URL}" | grep 'slotsPerEpoch:' | sed -e 's/^[[:space:]]*//' | sed -e 's/slotsPerEpoch: //')

echo "------------------------------------------------------"
echo "REST_PORT         : ${REST_PORT}"
echo "SOURCE_SK         : ${SOURCE_SK}"
echo "SOURCE_PK         : ${SOURCE_PK}"
echo "SRC_ADDR          : ${SRC_ADDR}"
echo "SOURCE_COUNTER    : ${SOURCE_COUNTER}"
echo "FEE_CONSTANT      : ${FEE_CONSTANT}"
echo "FEE_COEFFICIENT   : ${FEE_COEFFICIENT}"
echo "FEE_CERTIFICATE   : ${FEE_CERTIFICATE}"
echo "BLOCK0_HASH       : ${BLOCK0_HASH}"
echo "MAX_TXS_PER_BLOCK : ${MAX_TXS_PER_BLOCK}"
echo "SLOT_DURATION     : ${SLOT_DURATION}"
echo "SLOTS_PER_EPOCH   : ${SLOTS_PER_EPOCH}"
echo "------------------------------------------------------"

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
    SRC_COUNTER_INCR=$((${SOURCE_COUNTER} + ${i} - 1))
    sendMoney ${list_of_source_addrs[$i]}
done

waitNewBlockCreated

NO_OF_MINED_TXS_INIT=$(getNoOfMinedTransactions)

###
#   3. run multiple-transactions-per-slot-incr script in parallel
###
START_TIME=`date +"%T"`;

bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[1]} | tee script1Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[2]} | tee script2Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[3]} | tee script3Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[4]} | tee script4Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[5]} | tee script5Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[6]} | tee script6Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[7]} | tee script7Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[8]} | tee script8Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[9]} | tee script9Logs.txt &
bash multiple-transactions-per-slot-incr.sh ${TOTAL_TRANSACTIONS_PER_SOURCE} ${REST_PORT} ${list_of_source_sks[10]} | tee script10Logs.txt &

wait
echo "all processes completed"
END_TIME=`date +"%T"`;

SEC1=`date +%s -d ${START_TIME}`
SEC2=`date +%s -d ${END_TIME}`

TX_TIME=$((${SEC2} - ${SEC1}))
NO_OF_MINED_TXS_FINAL=$(getNoOfMinedTransactions)
NO_OF_MINED_TXS=$((${NO_OF_MINED_TXS_FINAL} - ${NO_OF_MINED_TXS_INIT}))

echo "===== total test time for sending txs (+ 1 new block): ${TX_TIME} secs = `date +%H:%M:%S -ud @${TX_TIME}`"
echo "===== sent transactions : $((${TOTAL_SOURCES} * ${TOTAL_TRANSACTIONS_PER_SOURCE}))"
echo "===== mined transactions: ${NO_OF_MINED_TXS}"
echo "===== TPS: $((${NO_OF_MINED_TXS} / ${TX_TIME}))"

