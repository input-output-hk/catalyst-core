var exec = require('cordova/exec');
var argscheck = require('cordova/argscheck');
var base64 = require('cordova/base64');

const NATIVE_CLASS_NAME = 'WalletPlugin';

const WALLET_RESTORE_ACTION_TAG = 'WALLET_RESTORE';
const WALLET_IMPORT_KEYS_TAG = 'WALLET_IMPORT_KEYS';
const WALLET_RETRIEVE_FUNDS_ACTION_TAG = 'WALLET_RETRIEVE_FUNDS';
const WALLET_TOTAL_FUNDS_ACTION_TAG = 'WALLET_TOTAL_FUNDS';
const WALLET_SPENDING_COUNTER_ACTION_TAG = 'WALLET_SPENDING_COUNTER';
const WALLET_ID_TAG = 'WALLET_ID';
const WALLET_CONVERT_ACTION_TAG = 'WALLET_CONVERT';
const WALLET_SET_STATE_ACTION_TAG = 'WALLET_SET_STATE';
const WALLET_VOTE_ACTION_TAG = 'WALLET_VOTE';
const WALLET_CONFIRM_TRANSACTION = 'WALLET_CONFIRM_TRANSACTION';
const CONVERSION_TRANSACTIONS_SIZE_ACTION_TAG = 'CONVERSION_TRANSACTIONS_SIZE';
const CONVERSION_TRANSACTIONS_GET_ACTION_TAG = 'CONVERSION_TRANSACTIONS_GET';
const CONVERSION_IGNORED_GET_ACTION_TAG = 'CONVERSION_IGNORED';
const PROPOSAL_NEW_PUBLIC_ACTION_TAG = 'PROPOSAL_NEW_PUBLIC';
const PROPOSAL_NEW_PRIVATE_ACTION_TAG = 'PROPOSAL_NEW_PRIVATE';
const WALLET_DELETE_ACTION_TAG = 'WALLET_DELETE';
const SETTINGS_DELETE_ACTION_TAG = 'SETTINGS_DELETE';
const CONVERSION_DELETE_ACTION_TAG = 'CONVERSION_DELETE';
const PROPOSAL_DELETE_ACTION_TAG = 'PROPOSAL_DELETE';
const WALLET_PENDING_TRANSACTIONS = 'WALLET_PENDING_TRANSACTIONS';
const PENDING_TRANSACTIONS_DELETE = 'PENDING_TRANSACTIONS_DELETE';
const PENDING_TRANSACTIONS_GET = 'PENDING_TRANSACTIONS_GET';
const PENDING_TRANSACTIONS_SIZE = 'PENDING_TRANSACTIONS_SIZE';
const SYMMETRIC_CIPHER_DECRYPT = 'SYMMETRIC_CIPHER_DECRYPT';
const SETTINGS_NEW = 'SETTINGS_NEW';
const SETTINGS_GET = 'SETTINGS_GET';
const FRAGMENT_ID = 'FRAGMENT_ID';
const TTL_FROM_DATE = "TTL_FROM_DATE";

const VOTE_PLAN_ID_LENGTH = 32;
const FRAGMENT_ID_LENGTH = 32;
const ED25519_EXTENDED_LENGTH = 64;

/**
 * THOUGHTS/TODO
 * add a more idiomatic abstraction on top of these primitive functions and expose that, something more similar to what wasm-bindgen does
 * I'm still not sure what javascript features can we use here (ES6, can we bring dependencies?, promises?)
*/

/**
 * wallet module.
 * @exports wallet-cordova-plugin.wallet
 */
var plugin = {
    /**
     * @callback pointerCallback
     * @param {string} ptr - callback that returns a pointer to a native object
     */

    /**
     * @callback errorCallback
     * @param {string} error - error description
     */

    /**
     * @callback SettingsCallback
     * @param {Settings} settings
     */

    /**
     * @callback BlockDateCallback
     * @param {BlockDate} date
     */

    /**
     * @typedef Settings
     * @type {object}
     * @property {Uint8Array} block0Hash
     * @property {Discrimination} discrimination
     * @property {Fees} fees
     */

    /**
     * @typedef Fees
     * @type {object}
     * @property {string} constant
     * @property {string} coefficient
     * @property {string} certificate
     * @property {string} certificatePoolRegistration
     * @property {string} certificateStakeDelegation
     * @property {string} certificateOwnerStakeDelegation
     * @property {string} certificateVotePlan
     * @property {string} certificateVoteCast
     */

    /**
     * @typedef TimeEra
     * @type {object}
     * @property {string} epochStart
     * @property {string} slotStart
     * @property {string} slotsPerEpoch
     */

    /**
     * @typedef BlockDate
     * @type {object}
     * @property {string} epoch
     * @property {string} slot
     */

    /**
     * @readonly
     * @enum {number}
     */
    Discrimination: {
        PRODUCTION: 0,
        TEST: 1
    },

    /**
     * @callback TransactionIdCallback
     * @param {Uint8Array} id
     */

    /**
     * @param {string} mnemonics a string with the mnemonic phrase
     * @param {pointerCallback} successCallback on success returns a pointer to a Wallet object
     * @param {errorCallback} errorCallback this function can fail if the mnemonics are invalid
     */
    walletRestore: function (mnemonics, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'walletRestore', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_RESTORE_ACTION_TAG, [mnemonics]);
    },

    /**
     * @param {Uint8Array} accountKeys a 64bytes array representing an Ed25519Extended private key
     * @param {Uint8Array} utxoKeys a contiguous array of Ed25519Extended private keys (64 bytes each)
     * @param {pointerCallback} successCallback on success returns a pointer to a Wallet object
     * @param {errorCallback} errorCallback if the input arrays are malformed
     */
    walletImportKeys: function (accountKeys, utxoKeys, successCallback, errorCallback) {
        argscheck.checkArgs('**ff', 'walletImportKeys', arguments);
        checkUint8Array({ name: 'accountKeys', testee: accountKeys, optLength: ED25519_EXTENDED_LENGTH });
        checkUint8Array({ name: 'utxoKeys', testee: utxoKeys });

        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_IMPORT_KEYS_TAG, [accountKeys.buffer, utxoKeys.buffer]);
    },

    /**
     * @param {string} ptr a pointer to a wallet obtained with walletRestore
     * @param {Uint8Array} block0 a byte array representing the block
     * @param {function} successCallback returns a pointer to the blockchain settings extracted from the block
     * @param {errorCallback} errorCallback this can fail if the block or the pointer are invalid
     */
    walletRetrieveFunds: function (ptr, block0, successCallback, errorCallback) {
        argscheck.checkArgs('s*ff', 'walletRetrieveFunds', arguments);
        checkUint8Array({ name: 'block0', testee: block0 });

        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_RETRIEVE_FUNDS_ACTION_TAG, [ptr, block0.buffer]);
    },

    /**
     * @param {string} ptr a pointer to a wallet obtained with walletRestore
     * @param {function} successCallback returns a number
     * @param {errorCallback} errorCallback description (TODO)
     */
    walletTotalFunds: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'walletTotalFunds', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_TOTAL_FUNDS_ACTION_TAG, [ptr]);
    },

    /**
     * @param {string} ptr a pointer to a wallet
     * @param {function} successCallback returns a number
     * @param {errorCallback} errorCallback this function should not fail
     */
    walletSpendingCounter: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'walletTotalFunds', arguments);

        const toNumber = function (arg) {
            successCallback(Number(arg));
        };

        exec(toNumber, errorCallback, NATIVE_CLASS_NAME, WALLET_SPENDING_COUNTER_ACTION_TAG, [ptr]);
    },

    /**
     * get the wallet id

     * This ID is the identifier to use against the blockchain/explorer to retrieve
     * the state of the wallet (counter, total value etc...)
     *
     * # Safety
     *
     * This function dereference raw pointers (wallet). Even though
     * the function checks if the pointers are null. Mind not to put random values
     * in or you may see unexpected behaviors
     *
     * @param {string} ptr a pointer to a Wallet object obtained with WalletRestore
     * @param {function} successCallback the return value is an ArrayBuffer, which has the binary representation of the account id.
     * @param {function} errorCallback this function may fail if the wallet pointer is null
     */
    walletId: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'walletId', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_ID_TAG, [ptr]);
    },

    /**
     *
     * update the wallet account state
     *
     * this is the value retrieved from any jormungandr endpoint that allows to query
     * for the account state. It gives the value associated to the account as well as
     * the counter.
     *
     * It is important to be sure to have an updated wallet state before doing any
     * transactions otherwise future transactions may fail to be accepted by any
     * nodes of the blockchain because of invalid signature state.
     *
     * # Errors
     *
     * this function may fail if the wallet pointer is null;
     * @param {string} ptr a pointer to a Wallet object obtained with WalletRestore
     * @param {number} value
     * @param {number} counter
     * @param {function} successCallback
     * @param {function} errorCallback
     *
     */
    walletSetState: function (ptr, value, counter, successCallback, errorCallback) {
        argscheck.checkArgs('snnff', 'walletSetState', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_SET_STATE_ACTION_TAG, [ptr, value, counter]);
    },

    /**
     *
     * Get a signed transaction with a vote of `choice` to the given proposal, ready to be sent to the network.
     *
     * # Errors
     *
     * this function may fail if if any of the pointers are is null;
     * @param {string} walletPtr a pointer to a Wallet object obtained with walletRestore
     * @param {string} settingsPtr a pointer to a Settings object obtained with walletRetrieveFunds
     * @param {string} proposalPtr a pointer to a Proposal object obtained with proposalNew
     * @param {number} choice a number between 0 and Proposal's numChoices - 1
     * @param {BlockDate} validUntil maximum date in which this fragment can be applied to the ledger
     * @param {function} successCallback on success the callback returns a byte array representing a transaction
     * @param {function} errorCallback can fail if the choice doesn't validate with the given proposal
     *
     */
    walletVote: function (walletPtr, settingsPtr, proposalPtr, choice, validUntil, successCallback, errorCallback) {
        argscheck.checkArgs('sssn*ff', 'walletVote', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_VOTE_ACTION_TAG, [walletPtr, settingsPtr, proposalPtr, choice, validUntil]);
    },

    /**
     * @param {string} walletPtr a pointer to a wallet obtained with walletRestore
     * @param {string} settingsPtr a pointer to a settings object obtained with walletRetrieveFunds
     * @param {BlockDate} validUntil maximum date in which this fragment can be applied to the ledger
     * @param {pointerCallback} successCallback returns a Conversion object
     * @param {errorCallback} errorCallback description (TODO)
     */
    walletConvert: function (walletPtr, settingsPtr, validUntil, successCallback, errorCallback) {
        argscheck.checkArgs('ss*ff', 'walletConvert', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_CONVERT_ACTION_TAG, [walletPtr, settingsPtr, validUntil]);
    },

    /**
     * @param {string} walletPtr a pointer to a wallet obtained with walletRestore
     * @param {pointerCallback} successCallback
     * @param {errorCallback} errorCallback
     */
    walletPendingTransactions: function (walletPtr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'walletPendingTransactions', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_PENDING_TRANSACTIONS, [walletPtr]);
    },

    /**
     * @param {string} ptr a pointer to a Conversion object obtained with walletConvert
     * @param {function} successCallback returns a number representing the number of transactions produced by the conversion
     * @param {errorCallback} errorCallback description (TODO)
     */
    pendingTransactionsSize: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('*ff', 'pendingTransactionsSize', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, PENDING_TRANSACTIONS_SIZE, [ptr]);
    },

    /**
     * @param {string} ptr a pointer to a PendingTransactions object obtained with walletPendingTransactions
     * @param {number} index an index (starting from 0). Use pendingTransactionsSize to get the upper bound
     * @param {TransactionIdCallback} successCallback callback that receives a transaction id in binary form
     * @param {errorCallback} errorCallback this function can fail if the index is out of range
     */
    pendingTransactionsGet: function (ptr, index, successCallback, errorCallback) {
        argscheck.checkArgs('*nff', 'pendingTransactionsGet', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, PENDING_TRANSACTIONS_GET, [ptr, index]);
    },

    /**
     * @param {string} walletPtr a pointer to a wallet obtained with walletRestore
     * @param {Uint8Array} transactionId the transaction id in bytes
     * @param {pointerCallback} successCallback
     * @param {errorCallback} errorCallback
     */
    walletConfirmTransaction: function (walletPtr, transactionId, successCallback, errorCallback) {
        argscheck.checkArgs('s*ff', 'walletConfirmTransaction', arguments);
        checkUint8Array({ name: 'transactionId', testee: transactionId, optLength: FRAGMENT_ID_LENGTH });

        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_CONFIRM_TRANSACTION, [walletPtr, transactionId.buffer]);
    },

    /**
     * @param {string} ptr a pointer to a Conversion object obtained with walletConvert
     * @param {function} successCallback returns a number representing the number of transactions produced by the conversion
     * @param {errorCallback} errorCallback description (TODO)
     */
    conversionTransactionsSize: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'conversionTransactionsSize', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, CONVERSION_TRANSACTIONS_SIZE_ACTION_TAG, [ptr]);
    },

    /**
     * @param {string} ptr a pointer to a Conversion object obtained with walletConvert
     * @param {number} index an index (starting from 0). Use conversionTransactionsSize to get the upper bound
     * @param {function} successCallback callback that receives a transaction in binary form
     * @param {errorCallback} errorCallback this function can fail if the index is out of range
     */
    conversionTransactionsGet: function (ptr, index, successCallback, errorCallback) {
        argscheck.checkArgs('snff', 'conversionTransactionsGet', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, CONVERSION_TRANSACTIONS_GET_ACTION_TAG, [ptr, index]);
    },

    /**
     * @param {string} ptr a pointer to a Conversion object obtained with walletConvert
     * @param {function} successCallback returns an object with ignored, and value properties
     * @param {errorCallback} errorCallback
     */
    conversionGetIgnored: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'conversionGetIgnored', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, CONVERSION_IGNORED_GET_ACTION_TAG, [ptr]);
    },

    /**
     * Get a proposal object, used to validate the vote on `walletVote`
     *
     * @param {Uint8Array} votePlanId a byte array of 32 elements that identifies the voteplan
     * @param {number} index the index of the proposal in the voteplan
     * @param {number} numChoices the number of choices of the proposal, used to validate the choice
     * @param {function} successCallback returns an object with ignored, and value properties
     * @param {errorCallback} errorCallback
     */
    proposalNewPublic: function (votePlanId, index, numChoices, successCallback, errorCallback) {
        argscheck.checkArgs('*nnff', 'proposalNewPublic', arguments);
        checkUint8Array({ name: 'votePlanId', testee: votePlanId, optLength: VOTE_PLAN_ID_LENGTH });

        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, PROPOSAL_NEW_PUBLIC_ACTION_TAG, [votePlanId.buffer, index, numChoices]);
    },

    /**
     * Get a proposal object, used to validate the vote on `walletVote`
     *
     * @param {Uint8Array} votePlanId a byte array of 32 elements that identifies the voteplan
     * @param {number} index the index of the proposal in the voteplan
     * @param {number} numChoices the number of choices of the proposal, used to validate the choice
     * @param {string} encryptionVoteKey bech32 string representing the
     * single key used to encrypt a vote, generated from the public keys
     * from all committee members
     * @param {function} successCallback returns an object with ignored, and value properties
     * @param {errorCallback} errorCallback
     */
    proposalNewPrivate: function (votePlanId, index, numChoices, encryptionVoteKey, successCallback, errorCallback) {
        argscheck.checkArgs('*nnsff', 'proposalNewPrivate', arguments);
        checkUint8Array({ name: 'votePlanId', testee: votePlanId, optLength: VOTE_PLAN_ID_LENGTH });

        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, PROPOSAL_NEW_PRIVATE_ACTION_TAG, [votePlanId.buffer, index, numChoices, encryptionVoteKey]);
    },

    /**
     * @param {Uint8Array} password the encryption password as bytes
     * @param {Uint8Array} ciphertext the encrypted bytes
     * @param {pointerCallback} successCallback on success returns a pointer to a Wallet object
     * @param {errorCallback} errorCallback this function can fail if the mnemonics are invalid
     */
    symmetricCipherDecrypt: function (password, ciphertext, successCallback, errorCallback) {
        argscheck.checkArgs('**ff', 'symmetricCipherDecrypt', arguments);
        checkUint8Array({ name: 'password', testee: password });
        checkUint8Array({ name: 'ciphertext', testee: ciphertext });

        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, SYMMETRIC_CIPHER_DECRYPT, [password.buffer, ciphertext.buffer]);
    },


    /**
     * @param {Uint8Array} block0Hash
     * @param {Discrimination} discrimination
     * @param {Fees} fees
     * @param {string} block0Date
     * @param {string} slotDuration
     * @param {TimeEra} era
     * @param {string} transactionMaxExpiryEpochs
     * @param {pointerCallback} successCallback
     * @param {errorCallback} errorCallback
     */
    settingsNew: function (
      block0Hash,
      discrimination,
      fees,
      block0Date,
      slotDuration,
      era,
      transactionMaxExpiryEpochs,
      successCallback,
      errorCallback
    ) {
      argscheck.checkArgs('*n*ss*sff', 'settingsNew', arguments);
      checkUint8Array({ name: 'block0Hash', testee: block0Hash });

      exec(successCallback, errorCallback, NATIVE_CLASS_NAME, SETTINGS_NEW, [
        block0Hash.buffer,
        discrimination,
        fees,
        block0Date,
        slotDuration,
        era,
        transactionMaxExpiryEpochs,
      ]);
    },

    /**
     * @param {string} settingsPtr
     * @param {SettingsCallback} settingsCallback
     * @param {errorCallback} errorCallback
     */
    settingsGet: function (settingsPtr, settingsCallback, errorCallback) {
        argscheck.checkArgs('sff', 'settingsGet', arguments);

        const decodeBase64 = function (arg) {
            arg.block0Hash = new Uint8Array(base64.toArrayBuffer(arg.block0Hash));
            settingsCallback(arg);
        };

        exec(decodeBase64, errorCallback, NATIVE_CLASS_NAME, SETTINGS_GET, [settingsPtr]);
    },

    /**
     * @param {Uint8Array} transaction
     * @param {TransactionIdCallback} successCallback
     * @param {errorCallback} errorCallback
     */
    fragmentId: function (transaction, successCallback, errorCallback) {
        argscheck.checkArgs('*ff', 'transactionId', arguments);
        checkUint8Array({ name: 'transaction', testee: transaction });

        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, FRAGMENT_ID, [transaction.buffer]);
    },

    /**
     * @param {string} settingsPtr
     * @param {number} unixEpoch
     * @param {BlockDateCallback} successCallback
     * @param {errorCallback} errorCallback
     */
    ttlFromDate: function (settingsPtr, unixEpoch, successCallback, errorCallback) {
        argscheck.checkArgs('snff', 'ttlFromDate', arguments);

        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, TTL_FROM_DATE, [settingsPtr, unixEpoch]);
    },

    /**
     * @param {string} ptr a pointer to a Wallet obtained with walletRestore
     * @param {function} successCallback  indicates success. Does not return anything.
     * @param {errorCallback} errorCallback
     */
    walletDelete: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'walletDelete', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_DELETE_ACTION_TAG, [ptr]);
    },

    /**
     * @param {string} ptr a pointer to a Settings object obtained with walletRetrieveFunds
     * @param {function} successCallback  indicates success. Does not return anything.
     * @param {errorCallback} errorCallback
     */
    settingsDelete: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'settingsDelete', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, SETTINGS_DELETE_ACTION_TAG, [ptr]);
    },

    /**
     * @param {string} ptr a pointer to a Conversion object obtained with walletConvert
     * @param {function} successCallback  indicates success. Does not return anything.
     * @param {errorCallback} errorCallback
     */
    conversionDelete: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'conversionDelete', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, CONVERSION_DELETE_ACTION_TAG, [ptr]);
    },

    /**
     * @param {string} ptr a pointer to a Proposal object obtained with proposalNew
     * @param {function} successCallback  indicates success. Does not return anything.
     * @param {errorCallback} errorCallback
     */
    proposalDelete: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('sff', 'proposalDelete', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, PROPOSAL_DELETE_ACTION_TAG, [ptr]);
    },

    /**
     * @param {string} ptr a pointer to a Proposal object obtained with proposalNew
     * @param {function} successCallback  indicates success. Does not return anything.
     * @param {errorCallback} errorCallback
     */
    pendingTransactionsDelete: function (ptr, successCallback, errorCallback) {
        argscheck.checkArgs('*ff', 'pendingTransactionsDelete', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, PENDING_TRANSACTIONS_DELETE, [ptr]);
    }
};

function checkUint8Array(arg) {
    var typeName = require('cordova/utils').typeName;
    var validType = arg.testee && typeName(arg.testee) === 'Uint8Array';
    if (!validType) {
        throw TypeError('expected ' + arg.name + ' to be of type Uint8Array');
    }

    var validLength = arg.optLength ? arg.testee.length === arg.optLength : true;
    if (!validLength) {
        throw TypeError('expected ' + arg.name + ' to have length ' + arg.optLength + ' found: ' + arg.testee.length);
    }
}

module.exports = plugin;
