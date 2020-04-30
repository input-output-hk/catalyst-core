const wasm = require('wallet-cordova-plugin.wasmModule');

function base64ToUint8Array (base64) {
    var binary_string = window.atob(base64);
    var len = binary_string.length;
    var bytes = new Uint8Array(len);
    for (var i = 0; i < len; i++) {
        bytes[i] = binary_string.charCodeAt(i);
    }
    return bytes;
}

function walletRestore (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string') {
        const mnemonics = opts[0];
        const password = '';
        try {
            const wallet = wasm.Wallet.recover(mnemonics, password);
            successCallback(wallet.ptr);
        } catch (err) {
            errorCallback(`couldn't recover wallet ${err}`);
        }
    } else {
        errorCallback('no mnemonics provided');
    }
}

function walletRetrieveFunds (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'number' && typeof (opts[1]) === 'string') {
        const walletPtr = opts[0];
        const base64Block = opts[1];

        const wallet = wasm.Wallet.__wrap(walletPtr);
        const block = base64ToUint8Array(base64Block);

        try {
            const settings = wallet.retrieve_funds(block);
            successCallback(settings.ptr);
        } catch (err) {
            errorCallback(`couldn't retrieve funds ${err}`);
        }
    } else {
        errorCallback('missing walletPtr or block');
    }
}

function walletTotalFunds (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'number') {
        const walletPtr = opts[0];
        const wallet = wasm.Wallet.__wrap(walletPtr);

        try {
            successCallback(wallet.total_value());
        } catch (err) {
            errorCallback(`couldn't get funds ${err}`);
        }
    } else {
        errorCallback('no pointer');
    }
}

function walletDelete (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'number') {
        const walletPtr = opts[0];
        wasm.Wallet.__wrap(walletPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

function settingsDelete (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'number') {
        const settingsPtr = opts[0];
        wasm.Settings.__wrap(settingsPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

const bindings = {
    WALLET_RESTORE: walletRestore,
    WALLET_RETRIEVE_FUNDS: walletRetrieveFunds,
    WALLET_TOTAL_FUNDS: walletTotalFunds,
    WALLET_DELETE: walletDelete,
    SETTINGS_DELETE: settingsDelete
};

// this is in done in order to make it work regardless of where the html file is located
const appPath = global.require('electron').remote.app.getAppPath();
const binaryPath = global.require('path').join(appPath, 'wallet_js_bg.wasm');
// TODO: we could probably do this async
const bytes = global.require('fs').readFileSync(binaryPath);

wasm(bytes)
    .then(() => { require('cordova/exec/proxy').add('WalletPlugin', bindings); });
