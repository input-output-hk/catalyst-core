var exec = require('cordova/exec');
var argscheck = require('cordova/argscheck');

const NATIVE_CLASS_NAME = 'WalletPlugin';

const WALLET_RESTORE_ACTION_TAG = 'WALLET_RESTORE';
const WALLET_RETRIEVE_FUNDS_ACTION_TAG = 'WALLET_RETRIEVE_FUNDS';
const WALLET_TOTAL_FUNDS_ACTION_TAG = 'WALLET_TOTAL_FUNDS';
const WALLET_ID_TAG = 'WALLET_ID';
const WALLET_CONVERT_ACTION_TAG = 'WALLET_CONVERT';
const WALLET_SET_STATE_ACTION_TAG = 'WALLET_SET_STATE';
const WALLET_VOTE_ACTION_TAG = 'WALLET_VOTE';
const CONVERSION_TRANSACTIONS_SIZE_ACTION_TAG = 'CONVERSION_TRANSACTIONS_SIZE';
const CONVERSION_TRANSACTIONS_GET_ACTION_TAG = 'CONVERSION_TRANSACTIONS_GET';
const CONVERSION_IGNORED_GET_ACTION_TAG = 'CONVERSION_IGNORED';
const PROPOSAL_NEW_ACTION_TAG = 'PROPOSAL_NEW';
const WALLET_DELETE_ACTION_TAG = 'WALLET_DELETE';
const SETTINGS_DELETE_ACTION_TAG = 'SETTINGS_DELETE';
const CONVERSION_DELETE_ACTION_TAG = 'CONVERSION_DELETE';
const PROPOSAL_DELETE_ACTION_TAG = 'PROPOSAL_DELETE';

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
     * @readonly
     * @enum {number}
     */
    PayloadType: {
        PUBLIC: 1
    },

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
     * @param {string} ptr a pointer to a wallet obtained with walletRestore
     * @param {Uint8Array} block0 a byte array representing the block
     * @param {function} successCallback returns a pointer to the blockchain settings extracted from the block
     * @param {errorCallback} errorCallback this can fail if the block or the pointer are invalid
     */
    walletRetrieveFunds: function (ptr, block0, successCallback, errorCallback) {
        argscheck.checkArgs('s*ff', 'walletRetrieveFunds', arguments);
        // cordova checkArgs doesn't support Uint8Array, so we use the * to let it pass and then check it ourselves
        if (require('cordova/utils').typeName(block0) === 'Uint8Array') {
            exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_RETRIEVE_FUNDS_ACTION_TAG, [ptr, block0.buffer]);
        } else {
            throw TypeError('expected block0 to be a Uint8Array in walletRetrieveFunds');
        }
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
     *
     * # Errors
     *
     * this function may fail if if any of the pointers are is null;
     * @param {string} walletPtr a pointer to a Wallet object obtained with WalletRestore
     * @param {string} settingsPtr
     * @param {string} proposalPtr
     * @param {number} choice
     * @param {function} successCallback
     * @param {function} errorCallback
     *
     */
    walletVote: function (walletPtr, settingsPtr, proposalPtr, choice, successCallback, errorCallback) {
        argscheck.checkArgs('sssnff', 'walletVote', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_VOTE_ACTION_TAG, [walletPtr, settingsPtr, proposalPtr, choice]);
    },

    /**
     * @param {string} walletPtr a pointer to a wallet obtained with walletRestore
     * @param {string} settingsPtr a pointer to a settings object obtained with walletRetrieveFunds
     * @param {pointerCallback} successCallback returns a Conversion object
     * @param {errorCallback} errorCallback description (TODO)
     */
    walletConvert: function (walletPtr, settingsPtr, successCallback, errorCallback) {
        argscheck.checkArgs('ssff', 'walletConvert', arguments);
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_CONVERT_ACTION_TAG, [walletPtr, settingsPtr]);
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
        argscheck.checkArgs('snff', 'conversionTransactiosGet', arguments);
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
     * @param {string} votePlanId
     * @param {PayloadType} payloadType
     * @param {number} index
     * @param {number} numChoices
     * @param {function} successCallback returns an object with ignored, and value properties
     * @param {errorCallback} errorCallback
     */
    proposalNew: function (votePlanId, payloadType, index, numChoices, successCallback, errorCallback) {
        argscheck.checkArgs('*nnnff', 'proposalNew', arguments);
        if (require('cordova/utils').typeName(votePlanId) === 'Uint8Array') {
            exec(successCallback, errorCallback, NATIVE_CLASS_NAME, PROPOSAL_NEW_ACTION_TAG, [votePlanId, payloadType, index, numChoices]);
        } else {
            throw TypeError('expected votePlanId to be a Uint8Array in proposalNew');
        }
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
    }
};

module.exports = plugin;
