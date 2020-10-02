package com.iohk.jormungandrwallet;

import java.util.concurrent.Callable;
import java.util.concurrent.Future;
import java.text.Normalizer;
import java.text.Normalizer.Form;

import com.iohk.jormungandrwallet.Settings;
import com.iohk.jormungandrwallet.Wallet;
import com.iohk.jormungandrwallet.Conversion;
import com.iohk.jormungandrwallet.Proposal;
import com.iohk.jormungandrwallet.SymmetricCipher;

import org.apache.cordova.CallbackContext;
import org.apache.cordova.CordovaInterface;
import org.apache.cordova.CordovaPlugin;
import org.apache.cordova.CordovaWebView;
import org.apache.cordova.CordovaArgs;
import org.json.JSONObject;
import org.json.JSONException;

import android.util.Base64;
import android.util.Log;

public class WalletPlugin extends CordovaPlugin {
    public static final String TAG = "WALLET";

    /**
     * Constructor.
     */
    public WalletPlugin() {
    }

    /**
     * Sets the context of the Command. This can then be used to do things like get
     * file paths associated with the Activity.
     *
     * @param cordova The context of the main Activity.
     * @param webView The CordovaWebView Cordova is running in.
     */
    public void initialize(final CordovaInterface cordova, final CordovaWebView webView) {
        super.initialize(cordova, webView);
        Log.d(TAG, "Initializing wallet plugin");
    }

    /**
     * Executes the request and returns PluginResult.
     *
     * @param action          The action to execute.
     * @param args            JSONArry of arguments for the plugin.
     * @param callbackContext The callback id used when calling back into
     *                        JavaScript.
     * @return True if the action was valid, false if not.
     */

    public boolean execute(final String action, final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        Log.d(TAG, "action: " + action);
        switch (action) {
            case "WALLET_RESTORE":
                walletRestore(args, callbackContext);
                break;
            case "WALLET_IMPORT_KEYS":
                walletImportKeys(args, callbackContext);
                break;
            case "SYMMETRIC_CIPHER_DECRYPT":
                symmetricCipherDecrypt(args, callbackContext);
                break;
            case "WALLET_RETRIEVE_FUNDS":
                walletRetrieveFunds(args, callbackContext);
                break;
            case "WALLET_TOTAL_FUNDS":
                walletTotalFunds(args, callbackContext);
                break;
            case "WALLET_ID":
                walletId(args, callbackContext);
                break;
            case "WALLET_SET_STATE":
                walletSetState(args, callbackContext);
                break;
            case "WALLET_VOTE":
                walletVote(args, callbackContext);
                break;
            case "WALLET_CONVERT":
                walletConvert(args, callbackContext);
                break;
            case "WALLET_PENDING_TRANSACTIONS":
                walletPendingTransactions(args, callbackContext);
                break;
            case "PENDING_TRANSACTIONS_SIZE":
                pendingTransactionsSize(args, callbackContext);
                break;
            case "PENDING_TRANSACTIONS_GET":
                pendingTransactionsGet(args, callbackContext);
                break;
            case "WALLET_CONFIRM_TRANSACTION":
                walletConfirmTransaction(args, callbackContext);
                break;
            case "CONVERSION_TRANSACTIONS_SIZE":
                conversionTransactionsSize(args, callbackContext);
                break;
            case "CONVERSION_TRANSACTIONS_GET":
                conversionTransactionsGet(args, callbackContext);
                break;
            case "CONVERSION_IGNORED":
                conversionIgnored(args, callbackContext);
                break;
            case "PROPOSAL_NEW_PUBLIC":
                proposalNewPublic(args, callbackContext);
                break;
            case "PROPOSAL_NEW_PRIVATE":
                proposalNewPrivate(args, callbackContext);
                break;
            case "WALLET_DELETE":
                walletDelete(args, callbackContext);
                break;
            case "SETTINGS_DELETE":
                settingsDelete(args, callbackContext);
                break;
            case "CONVERSION_DELETE":
                conversionDelete(args, callbackContext);
                break;
            case "PROPOSAL_DELETE":
                proposalDelete(args, callbackContext);
                break;
            case "PENDING_TRANSACTIONS_DELETE":
                pendingDelete(args, callbackContext);
                break;
            default:
                return false;
        }

        return true;
    }

    private void walletRestore(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final String mnemonics = args.getString(0);

        cordova.getThreadPool().execute(new Runnable() {
            public void run() {
                try {
                    final String normalized = Normalizer.normalize(mnemonics, Form.NFKD);
                    final long walletPtr = Wallet.recover(normalized);
                    callbackContext.success(Long.toString(walletPtr));
                } catch (final Exception e) {
                    callbackContext.error(e.getMessage());
                }
            }
        });
    }

    private void walletImportKeys(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final byte[] accountKey = args.getArrayBuffer(0);
        final byte[] utxoKeys = args.getArrayBuffer(1);

        cordova.getThreadPool().execute(new Runnable() {
            public void run() {
                try {
                    final long walletPtr = Wallet.importKeys(accountKey, utxoKeys);
                    Log.d(TAG, Long.toString(walletPtr));
                    callbackContext.success(Long.toString(walletPtr));
                } catch (final Exception e) {
                    callbackContext.error(e.getMessage());
                }
            }
        });
    }

    private void symmetricCipherDecrypt(final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        final byte[] password = args.getArrayBuffer(0);
        final byte[] ciphertext = args.getArrayBuffer(1);

        try {
            final byte[] decrypted = SymmetricCipher.decrypt(password, ciphertext);
            callbackContext.success(decrypted);
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletRetrieveFunds(final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        final Long walletPtr = args.getLong(0);
        final byte[] block0 = args.getArrayBuffer(1);

        cordova.getThreadPool().execute(new Runnable() {
            public void run() {
                try {
                    final long settingsPtr = Wallet.initialFunds(walletPtr, block0);
                    callbackContext.success(Long.toString(settingsPtr));
                } catch (final Exception e) {
                    callbackContext.error(e.getMessage());
                }
            }
        });
    }

    private void walletTotalFunds(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long walletPtr = args.getLong(0);

        try {
            final int value = Wallet.totalValue(walletPtr);
            callbackContext.success(Integer.toString(value));
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletSetState(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long walletPtr = args.getLong(0);
        final Long value = args.getLong(1);
        final Long counter = args.getLong(2);

        try {
            Wallet.setState(walletPtr, value, counter);
            callbackContext.success();
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletVote(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long wallet = args.getLong(0);
        final Long settings = args.getLong(1);
        final Long proposal = args.getLong(2);
        final Integer choice = args.getInt(3);

        try {
            final byte[] tx = Wallet.voteCast(wallet, settings, proposal, choice);
            callbackContext.success(tx);
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletPendingTransactions(final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        final Long wallet = args.getLong(0);

        try {
            Long pendingTransactions = Wallet.pendingTransactions(wallet);
            callbackContext.success(Long.toString(pendingTransactions));
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void pendingTransactionsSize(final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        final Long pendingTransactionsPtr = args.getLong(0);

        try {
            final int size = PendingTransactions.len(pendingTransactionsPtr);
            callbackContext.success(Integer.toString(size));
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void pendingTransactionsGet(final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        final Long pendingTransactionsPtr = args.getLong(0);
        final int index = args.getInt(1);

        try {
            final byte[] transaction = PendingTransactions.get(pendingTransactionsPtr, index);
            callbackContext.success(transaction);
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletConfirmTransaction(final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        final Long wallet = args.getLong(0);
        final byte[] fragmentId = args.getArrayBuffer(1);

        try {
            Wallet.confirmTransaction(wallet, fragmentId);
            callbackContext.success();
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletConvert(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long walletPtr = args.getLong(0);
        final Long settingsPtr = args.getLong(1);

        cordova.getThreadPool().execute(new Runnable() {
            public void run() {
                try {
                    final long conversionPtr = Wallet.convert(walletPtr, settingsPtr);
                    callbackContext.success(Long.toString(conversionPtr));
                } catch (final Exception e) {
                    callbackContext.error(e.getMessage());
                }
            }
        });
    }

    private void conversionTransactionsSize(final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        final Long conversionsPtr = args.getLong(0);

        try {
            final int size = Conversion.transactionsSize(conversionsPtr);
            callbackContext.success(Integer.toString(size));
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void conversionTransactionsGet(final CordovaArgs args, final CallbackContext callbackContext)
            throws JSONException {
        final Long conversionsPtr = args.getLong(0);
        final int index = args.getInt(1);

        try {
            final byte[] transaction = Conversion.transactionsGet(conversionsPtr, index);
            callbackContext.success(transaction);
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void conversionIgnored(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long conversionPtr = args.getLong(0);

        try {
            Conversion.ignored(conversionPtr, new Conversion.IgnoredCallback() {
                @Override
                public void call(final long value, final long ignored) {
                    try {
                        final JSONObject json = new JSONObject().put("value", value).put("ignored", ignored);
                        callbackContext.success(json);
                    } catch (final JSONException e) {
                        throw new RuntimeException(e);
                    }
                }
            });
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void proposalNewPublic(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final byte[] votePlanId = args.getArrayBuffer(0);
        final Integer index = args.getInt(1);
        final Integer numChoices = args.getInt(2);

        try {
            long ptr = Proposal.withPublicPayload(votePlanId, index, numChoices);
            callbackContext.success(Long.toString(ptr));
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void proposalNewPrivate(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final byte[] votePlanId = args.getArrayBuffer(0);
        final Integer index = args.getInt(1);
        final Integer numChoices = args.getInt(2);

        try {
            long ptr = Proposal.withPublicPayload(votePlanId, index, numChoices);
            callbackContext.success(Long.toString(ptr));
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletId(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long walletPtr = args.getLong(0);

        try {
            final byte[] id = Wallet.id(walletPtr);
            callbackContext.success(id);
        } catch (final Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletDelete(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long walletPtr = args.getLong(0);

        Wallet.delete(walletPtr);
        callbackContext.success();
    }

    private void settingsDelete(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long settingsPtr = args.getLong(0);

        Settings.delete(settingsPtr);
        callbackContext.success();
    }

    private void conversionDelete(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long conversionPtr = args.getLong(0);

        Conversion.delete(conversionPtr);
        callbackContext.success();
    }

    private void proposalDelete(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long proposalPtr = args.getLong(0);

        Proposal.delete(proposalPtr);
        callbackContext.success();
    }

    private void pendingDelete(final CordovaArgs args, final CallbackContext callbackContext) throws JSONException {
        final Long pendingPtr = args.getLong(0);

        PendingTransactions.delete(pendingPtr);
        callbackContext.success();
    }
}