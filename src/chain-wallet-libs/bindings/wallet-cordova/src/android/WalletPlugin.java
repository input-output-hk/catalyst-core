/*
 TODO: maybe add license
*/
package com.iohk.jormungandrwallet;

import java.io.UnsupportedEncodingException;
import java.util.TimeZone;
import java.util.concurrent.Future;
import java.util.concurrent.Callable;

import org.apache.cordova.CordovaWebView;
import org.apache.cordova.CallbackContext;
import org.apache.cordova.CordovaPlugin;
import org.apache.cordova.CordovaInterface;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import android.util.Base64;
import android.util.Log;

import com.iohk.jormungandrwallet.Wallet;
import com.iohk.jormungandrwallet.Settings;

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
    public void initialize(CordovaInterface cordova, CordovaWebView webView) {
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
    public boolean execute(String action, JSONArray args, CallbackContext callbackContext) throws JSONException {
        Log.d(TAG, "action: " + action);
        Log.d(TAG, "arguments: " + args.toString());
        switch (action) {
            case "WALLET_RESTORE":
                walletRestore(args, callbackContext);
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
            case "WALLET_DELETE":
                walletDelete(args, callbackContext);
                break;
            case "SETTINGS_DELETE":
                settingsDelete(args, callbackContext);
                break;
            default:
                return false;
        }

        return true;
    }

    private void walletRestore(JSONArray args, CallbackContext callbackContext) throws JSONException {
        String mnemonics = args.getString(0);

        cordova.getThreadPool().execute(new Runnable() {
            public void run() {
                try {
                    long walletPtr = Wallet.recover(mnemonics);
                    callbackContext.success(Long.toString(walletPtr));
                } catch (Exception e) {
                    callbackContext.error(e.getMessage());
                }
            }
        });
    }

    private void walletRetrieveFunds(JSONArray args, CallbackContext callbackContext) throws JSONException {
        Long walletPtr = args.getLong(0);
        String block0 = args.getString(1);

        Future<byte[]> future = cordova.getThreadPool().submit(new Callable<byte[]>() {
            public byte[] call() throws IllegalArgumentException {
                return Base64.decode(block0.getBytes(java.nio.charset.StandardCharsets.UTF_16LE), Base64.DEFAULT);
            }
        });

        cordova.getThreadPool().execute(new Runnable() {
            public void run() {
                try {
                    byte[] block0_decoded = future.get();
                    long settingsPtr = Wallet.initialFunds(walletPtr, block0_decoded);
                    callbackContext.success(Long.toString(settingsPtr));
                } catch (Exception e) {
                    callbackContext.error(e.getMessage());
                }
            }
        });
    }

    private void walletTotalFunds(JSONArray args, CallbackContext callbackContext) throws JSONException {
        Long walletPtr = args.getLong(0);

        try {
            int value = Wallet.totalValue(walletPtr);
            callbackContext.success(Integer.toString(value));
        } catch (Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletId(JSONArray args, CallbackContext callbackContext) throws JSONException {
        Long walletPtr = args.getLong(0);

        try {
            byte[] id = Wallet.id(walletPtr);
            callbackContext.success(id);
        } catch (Exception e) {
            callbackContext.error(e.getMessage());
        }
    }

    private void walletDelete(JSONArray args, CallbackContext callbackContext) throws JSONException {
        Long walletPtr = args.getLong(0);

        Wallet.delete(walletPtr);
        callbackContext.success("");
    }

    private void settingsDelete(JSONArray args, CallbackContext callbackContext) throws JSONException {
        Long settingsPtr = args.getLong(0);

        Settings.delete(settingsPtr);
        callbackContext.success("");
    }
}
