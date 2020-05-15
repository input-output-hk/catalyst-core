var exec = require('cordova/exec');

const NATIVE_CLASS_NAME = 'WalletPlugin';

const WALLET_RESTORE_ACTION_TAG = 'WALLET_RESTORE';
const WALLET_RETRIEVE_FUNDS_ACTION_TAG = 'WALLET_RETRIEVE_FUNDS';
const WALLET_TOTAL_FUNDS_ACTION_TAG = 'WALLET_TOTAL_FUNDS';
const WALLET_ID_TAG = 'WALLET_ID';
const WALLET_CONVERT_ACTION_TAG = 'WALLET_CONVERT';
const CONVERSION_TRANSACTIONS_SIZE_ACTION_TAG = 'CONVERSION_TRANSACTIONS_SIZE';
const CONVERSION_TRANSACTIONS_GET_ACTION_TAG = 'CONVERSION_TRANSACTIONS_GET';
const WALLET_DELETE_ACTION_TAG = 'WALLET_DELETE';
const SETTINGS_DELETE_ACTION_TAG = 'SETTINGS_DELETE';
const CONVERSION_DELETE_ACTION_TAG = 'CONVERSION_DELETE';

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
     * @param {number} ptr - callback that returns a pointer to a native object
     */

    /**
     * @callback errorCallback
     * @param {string} error - error description
     */

    /**
     * @param {string} mnemonics a string with the mnemonic phrase
     * @param {pointerCallback} successCallback on success returns a pointer to a Wallet object
     * @param {errorCallback} errorCallback this function can fail if the mnemonics are invalid
     */
    walletRestore: function (mnemonics, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_RESTORE_ACTION_TAG, [mnemonics]);
    },

    /**
     * @param {Uint8Array} block0 a byte array representing the block
     * @param {function} successCallback returns a pointer to the blockchain settings extracted from the block
     * @param {errorCallback} errorCallback this can fail if the block or the pointer are invalid
     */
    walletRetrieveFunds: function (ptr, block0, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_RETRIEVE_FUNDS_ACTION_TAG, [ptr, block0.buffer]);
    },

    /**
     * @param {number} ptr a pointer to a wallet obtained with walletRestore
     * @param {function} successCallback returns a number
     * @param {errorCallback} errorCallback description (TODO)
     */
    walletTotalFunds: function (ptr, successCallback, errorCallback) {
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
     * @param {number} ptr a pointer to a Wallet object obtained with WalletRestore
     * @param {function} successCallback the return value is an ArrayBuffer, which has the binary representation of the account id.
     * @param {function} errorCallback this function may fail if the wallet pointer is null
     */
    walletId: function (ptr, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_ID_TAG, [ptr]);
    },

    /**
     * @param {number} walletPtr a pointer to a wallet obtained with walletRestore
     * @param {number} settingsPtr a pointer to a settings object obtained with walletRetrieveFunds
     * @param {pointerCallback} successCallback returns a Conversion object
     * @param {errorCallback} errorCallback description (TODO)
     */
    walletConvert: function (walletPtr, settingsPtr, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_CONVERT_ACTION_TAG, [walletPtr, settingsPtr]);
    },

    /**
     * @param {number} ptr a pointer to a Conversion object obtained with walletConvert
     * @param {function} successCallback returns a number representing the number of transactions produced by the conversion
     * @param {errorCallback} errorCallback description (TODO)
     */
    conversionTransactionsSize: function (ptr, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, CONVERSION_TRANSACTIONS_SIZE_ACTION_TAG, [ptr]);
    },

    /**
     * @param {number} ptr a pointer to a Conversion object obtained with walletConvert
     * @param {number} index an index (starting from 0). Use conversionTransactionsSize to get the upper bound
     * @param {function} successCallback callback that receives a transaction in binary form
     * @param {errorCallback} errorCallback this function can fail if the index is out of range
     */
    conversionTransactionsGet: function (ptr, index, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, CONVERSION_TRANSACTIONS_GET_ACTION_TAG, [ptr, index]);
    },

    /**
     * @param {number} walletPtr a pointer to a Wallet obtained with walletRestore
     * @param {function} successCallback  indicates success. Does not return anything.
     * @param {errorCallback} errorCallback
     */
    walletDelete: function (ptr, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, WALLET_DELETE_ACTION_TAG, [ptr]);
    },

    /**
     * @param {number} ptr a pointer to a Settings object obtained with walletRetrieveFunds
     * @param {function} successCallback  indicates success. Does not return anything.
     * @param {errorCallback} errorCallback
     */
    settingsDelete: function (ptr, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, SETTINGS_DELETE_ACTION_TAG, [ptr]);
    },

    /**
     * @param {number} ptr a pointer to a Conversion object obtained with walletConvert
     * @param {function} successCallback  indicates success. Does not return anything.
     * @param {errorCallback} errorCallback
     */
    conversionDelete: function (ptr, successCallback, errorCallback) {
        exec(successCallback, errorCallback, NATIVE_CLASS_NAME, CONVERSION_DELETE_ACTION_TAG, [ptr]);
    }
};

module.exports = plugin;
