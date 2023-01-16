#!/bin/sh

# Disclaimer:
#
#  The following use of shell script is for demonstration and understanding
#  only, it should *NOT* be used at scale or for any sort of serious
#  deployment, and is solely used for learning how the node and blockchain
#  works, and how to interact with everything.
#
#  It also assumes that `jcli` is in the same folder with the script.
#
#  This script is sending a number of transactions from a source account (that needs to have enough funds) to
#  a new account address.

### CONFIGURATION
CLI="./jcli"
COLORS=1
ADDRTYPE="--testing"
INITIAL_TIP=""
TX_COUNTER_SAME_SLOT="aa"
INITIAL_SRC_COUNTER=0
INITIAL_SOURCE_COUNTER=0
TIMEOUT_NO_OF_BLOCKS=100

if [ $# -ne 3 ]; then
  echo "usage: $0 <NO-OF-TRANSACTIONS> <REST-LISTEN-PORT> <ACCOUNT-SK>"
  echo "    <NO-OF-TRANSACTIONS> Number of transactions to be sent from Faucet to Account"
  echo "    <REST-LISTEN-PORT>   The REST Listen Port set in node-config.yaml file (EX: 3101)"
  echo "    <ACCOUNT-SK>         The Secret key of the Source Account address (for transactions)"
  exit 1
fi

NO_OF_TRANSACTIONS=$1
REST_PORT=$2
SOURCE_SK=$3

REST_URL="http://127.0.0.1:${REST_PORT}/api"

FEE_CONSTANT=$($CLI rest v0 settings get --host "${REST_URL}" | grep 'constant:' | sed -e 's/^[[:space:]]*//' | sed -e 's/constant: //')
FEE_COEFFICIENT=$($CLI rest v0 settings get --host "${REST_URL}" | grep 'coefficient:' | sed -e 's/^[[:space:]]*//' | sed -e 's/coefficient: //')
FEE_CERTIFICATE=$($CLI rest v0 settings get --host "${REST_URL}" | grep 'certificate:' | sed -e 's/^[[:space:]]*//' | sed -e 's/certificate: //')
BLOCK0_HASH=$($CLI rest v0 settings get --host "${REST_URL}" | grep 'block0Hash:' | sed -e 's/^[[:space:]]*//' | sed -e 's/block0Hash: //')
MAX_TXS_PER_BLOCK=$($CLI rest v0 settings get --host "${REST_URL}" | grep 'maxTxsPerBlock:' | sed -e 's/^[[:space:]]*//' | sed -e 's/maxTxsPerBlock: //')
SLOT_DURATION=$($CLI rest v0 settings get --host "${REST_URL}" | grep 'slotDuration:' | sed -e 's/^[[:space:]]*//' | sed -e 's/slotDuration: //')
SLOTS_PER_EPOCH=$($CLI rest v0 settings get --host "${REST_URL}" | grep 'slotsPerEpoch:' | sed -e 's/^[[:space:]]*//' | sed -e 's/slotsPerEpoch: //')

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
    echo $($CLI rest v0 tip get --host "${REST_URL}")
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

getAccountValue() {
    echo $($CLI rest v0 account get $1 --host "${REST_URL}" | grep 'value: ' | awk -F'value: ' '{print $2}')
}

getNoOfMinedTransactions() {
    echo $($CLI rest v0 message logs --host "${REST_URL}" | tr ' ' '\n' | grep 'InABlock:' | wc -l)
}

getTotalNoOfMesageLogs() {
    echo $($CLI rest v0 message logs --host "${REST_URL}" | tr ' ' '\n' | grep 'fragment_id:' | wc -l)
}

compareBalances() {
#    $1 = expected value
#    $2 = actual value
    if [[ $1 == $2 ]]; then
      echo "  ###OK; Correct Balance; $1 = $2"
    else
      echo " !!!!! ERROR: Actual Balance is different than expected; Expected: $1  vs  Actual: $2"
      exit 2
    fi
}

sendMoney() {
    if [[ $# -ne 2 ]]; then
        echo "usage: $0 <DST_ADDRESS> <AMOUNT>"
        echo "    <DST_ADDRESS>   Address to send amount of money to"
        echo "    <AMOUNT>        Amount in lovelace"
        exit 1
    fi

    SOURCE_ADDRESS=$($CLI address account ${ADDRTYPE} ${SOURCE_PK})
    DESTINATION_ADDRESS="$1"
    DESTINATION_AMOUNT="$2"

    # Account 1 pays for the transaction fee
    TX_AMOUNT=$((${DESTINATION_AMOUNT} + ${FEE_CONSTANT} + $((2 * ${FEE_COEFFICIENT}))))

    STAGING_FILE="acc_staging.$$.transaction"

    # increase the SOURCE_COUNTER with TX_COUNTER_SAME_SLOT if Account1 initiates more than 1 transaction in the same slot
    # TX_COUNTER_SAME_SLOT = the number of transactions initiated and sent by Account1 in the same slot (based on TIP)
    ACTUAL_TIP=$(getTip)
    SRC_COUNTER=$( $CLI rest v0 account get "${SOURCE_ADDRESS}" --host "${REST_URL}" | grep '^counter:' | sed -e 's/counter: //' )
#    echo "  ===== SRC_COUNTER: ${SRC_COUNTER}"

    if [[ ${ACTUAL_TIP} == ${INITIAL_TIP} ]]; then
#        echo "ACTUAL_TIP == INITIAL_TIP"
        if [[ ${SRC_COUNTER} -ne ${INITIAL_SRC_COUNTER} ]]; then
            echo "  == New block created after getting ACTUAL_TIP but before getting SRC_COUNTER"
            TX_COUNTER_SAME_SLOT=0
        else
            let TX_COUNTER_SAME_SLOT=TX_COUNTER_SAME_SLOT+1
        fi
    else
#        echo "ACTUAL_TIP != INITIAL_TIP"
        if [[ (${SRC_COUNTER} -ne ${INITIAL_SOURCE_COUNTER}) || (${TX_COUNTER_SAME_SLOT} == "aa") ]]; then
            TX_COUNTER_SAME_SLOT=0
        else
            TX_COUNTER_SAME_SLOT=1
        fi
        INITIAL_TIP=${ACTUAL_TIP}
    fi

    SOURCE_COUNTER=$((${SRC_COUNTER} + ${TX_COUNTER_SAME_SLOT}))
#    echo "  ===== SOURCE_COUNTER        : ${SOURCE_COUNTER}"
#    echo "  ===== TX_COUNTER_SAME_SLOT  : ${TX_COUNTER_SAME_SLOT}"

    INITIAL_SRC_COUNTER=${SRC_COUNTER}
    INITIAL_SOURCE_COUNTER=${SOURCE_COUNTER}
#    echo "  ===== INITIAL_SRC_COUNTER   : ${INITIAL_SRC_COUNTER}"
#    echo "  ===== INITIAL_SOURCE_COUNTER: ${INITIAL_SOURCE_COUNTER}"

    # Create the transaction
    $CLI transaction new --staging ${STAGING_FILE}
    $CLI transaction add-account "${SOURCE_ADDRESS}" "${TX_AMOUNT}" --staging "${STAGING_FILE}"
    $CLI transaction add-output "${DESTINATION_ADDRESS}" "${DESTINATION_AMOUNT}" --staging "${STAGING_FILE}"
    $CLI transaction finalize --staging ${STAGING_FILE}

    TRANSACTION_DATA=$($CLI transaction data-for-witness --staging ${STAGING_FILE})

    # Create the witness for the 1 input (add-account) and add it
    SRC_WITNESS_SECRET_FILE="witness.secret.$$"
    SRC_WITNESS_OUTPUT_FILE="witness.out.$$"

    printf "${SOURCE_SK}" > ${SRC_WITNESS_SECRET_FILE}

    $CLI transaction make-witness ${TRANSACTION_DATA} \
        --genesis-block-hash ${BLOCK0_HASH} \
        --type "account" --account-spending-counter "${SOURCE_COUNTER}" \
        ${SRC_WITNESS_OUTPUT_FILE} ${SRC_WITNESS_SECRET_FILE}
    $CLI transaction add-witness ${SRC_WITNESS_OUTPUT_FILE} --staging "${STAGING_FILE}"

    # Finalize the transaction and send it
    $CLI transaction seal --staging "${STAGING_FILE}"
    tx_hash=$($CLI transaction to-message --staging "${STAGING_FILE}" | $CLI rest v0 message post --host "${REST_URL}")

    echo "${tx_hash}" >> ${TX_HISTORY}

    rm ${STAGING_FILE} ${SRC_WITNESS_SECRET_FILE} ${SRC_WITNESS_OUTPUT_FILE}
}

######################## START TEST ########################
TX_VALUE=100
SRC_TX_VALUE=$((${TX_VALUE} + ${FEE_CONSTANT} + $((2 * ${FEE_COEFFICIENT}))))
REQUIRED_SRC_BALANCE=$((${NO_OF_TRANSACTIONS} * ${SRC_TX_VALUE}))

SOURCE_PK=$(echo ${SOURCE_SK} | $CLI key to-public)
SRC_ADDR=$($CLI address account ${ADDRTYPE} ${SOURCE_PK})

SRC_BALANCE_INIT=$(getAccountValue ${SRC_ADDR})
SOURCE_COUNTER=$( $CLI rest v0 account get "${SRC_ADDR}" --host "${REST_URL}" | grep '^counter:' | sed -e 's/counter: //' )
if [[ ${SOURCE_COUNTER} -gt 0 ]]; then
    SRC_BALANCE_INIT=$(getAccountValue ${SRC_ADDR})
fi
echo "SOURCE_SK         : ${SOURCE_SK}"
echo "SOURCE_PK         : ${SOURCE_PK}"
echo "SRC_ADDR          : ${SRC_ADDR}"
echo "SRC_BALANCE_INIT  : ${SRC_BALANCE_INIT}"
echo "SOURCE_COUNTER    : ${SOURCE_COUNTER}"

echo "- Required funds: ${REQUIRED_SRC_BALANCE}  vs  Available funds: ${SRC_BALANCE_INIT}"
if [ ${REQUIRED_SRC_BALANCE} -gt ${SRC_BALANCE_INIT} ]; then
  echo "!!!!! WARNING: Source Account's funds are insufficient for all required transactions !!!!!"
fi

echo "Create a destination Account address (RECEIVER_ADDR)"
DST_SK=$($CLI key generate --type=ed25519extended)
DST_PK=$(echo ${DST_SK} | $CLI key to-public)
DST_ADDR=$($CLI address account ${ADDRTYPE} ${DST_PK})
echo "DST_SK  : ${DST_SK}"
echo "DST_PK  : ${DST_PK}"
echo "DST_ADDR: ${DST_ADDR}"
DST_BALANCE_INIT=0

echo "read actual state of the message logs"
noOfMinedTxs_init=$(getNoOfMinedTransactions)
noOfTotalMessages_init=$(getTotalNoOfMesageLogs)

echo "noOfMinedTxs_init     : ${noOfMinedTxs_init}"
echo "noOfTotalMessages_init: ${noOfTotalMessages_init}"

##
# 1. create multiple transactions from Source to Destination Account and check balances at the end
##

BALANCE_HISTORY="balance_history.txt"
TX_HISTORY="tx_history.txt"

if [ -e ${BALANCE_HISTORY} ]; then
  rm ${BALANCE_HISTORY}
  touch ${BALANCE_HISTORY}
fi

if [ -e ${TX_HISTORY} ]; then
  rm ${TX_HISTORY}
  touch ${TX_HISTORY}
fi

SRC_BALANCE_INIT=$(getAccountValue ${SRC_ADDR})

echo "SRC_BALANCE_INIT: ${SRC_BALANCE_INIT}" >> ${BALANCE_HISTORY}
echo "DST_BALANCE_INIT: ${DST_BALANCE_INIT}" >> ${BALANCE_HISTORY}

SENT_VALUE=0
START_TIME="`date +%Y%m%d%H%M%S`";
for i in `seq 1 ${NO_OF_TRANSACTIONS}`;
do
    echo "##Transaction No: ${BLUE}$i${WHITE}; Value: $TX_VALUE"
    sendMoney ${DST_ADDR} ${TX_VALUE}
    SENT_VALUE=$((${SENT_VALUE} + ${TX_VALUE}))
done

END_TIME1="`date +%Y%m%d%H%M%S`";

waitNewBlockCreated

END_TIME2="`date +%Y%m%d%H%M%S`";

echo "=================Check the message logs (after 1 block)=================="
noOfMinedTxs_final=$(getNoOfMinedTransactions)
noOfTotalMessages_final=$(getTotalNoOfMesageLogs)

echo "total txs sent in current test            : ${NO_OF_TRANSACTIONS}"
echo "total txs mined in current test           : $((${noOfMinedTxs_final} - ${noOfMinedTxs_init}))"
echo "total fragments created in current test   : $((${noOfTotalMessages_final} - ${noOfTotalMessages_init}))"
echo "total time for sending transactions       : $((${END_TIME1} - ${START_TIME})) seconds"
echo "total test time (waiting 1 new block)     : $((${END_TIME2} - ${START_TIME})) seconds"

echo "=================Check the message logs (after 2 blocks)=================="
waitNewBlockCreated
END_TIME3="`date +%Y%m%d%H%M%S`";

echo "total txs sent in current test            : ${NO_OF_TRANSACTIONS}"
echo "total txs mined in current test           : $((${noOfMinedTxs_final} - ${noOfMinedTxs_init}))"
echo "total fragments created in current test   : $((${noOfTotalMessages_final} - ${noOfTotalMessages_init}))"
echo "total time for sending transactions       : $((${END_TIME1} - ${START_TIME})) seconds"
echo "total test time (waiting 2 new blocks)    : $((${END_TIME3} - ${START_TIME})) seconds"

echo "=================Check Destination Account's balance=================="
SRC_BALANCE_FINAL=$(getAccountValue ${SRC_ADDR})
DST_BALANCE_FINAL=$(getAccountValue ${DST_ADDR})

echo "SRC_BALANCE_FINAL: ${SRC_BALANCE_FINAL}" >> ${BALANCE_HISTORY}
echo "DST_BALANCE_FINAL: ${DST_BALANCE_FINAL}" >> ${BALANCE_HISTORY}
echo "SRC_BALANCE_DIFF: $((${SRC_BALANCE_INIT} - ${SRC_BALANCE_FINAL}))" >> ${BALANCE_HISTORY}
echo "DST_BALANCE_DIFF: $((${DST_BALANCE_FINAL} - ${DST_BALANCE_INIT}))" >> ${BALANCE_HISTORY}
echo "Sent transactions (based on consumed funds): $(($((${SRC_BALANCE_INIT} - ${SRC_BALANCE_FINAL})) / ${SRC_TX_VALUE}))" >> ${BALANCE_HISTORY}
echo "Received transactions (based on received funds): $(($((${DST_BALANCE_FINAL} - ${DST_BALANCE_INIT})) / ${TX_VALUE}))" >> ${BALANCE_HISTORY}

ACTUAL_DST_VALUE=$(getAccountValue ${DST_ADDR})
EXPECTED_DST_VALUE=$((${DST_BALANCE_INIT} + ${SENT_VALUE}))
compareBalances ${ACTUAL_DST_VALUE} ${EXPECTED_DST_VALUE}
