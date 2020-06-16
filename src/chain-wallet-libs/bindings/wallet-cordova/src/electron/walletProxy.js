const wasm = require('wallet-cordova-plugin.wasmModule');
const bindings = require('wallet-cordova-plugin.bindings');

// this is in done in order to make it work regardless of where the html file is located
const appPath = global.require('electron').remote.app.getAppPath();
const binaryPath = global.require('path').join(appPath, 'wallet_js_bg.wasm');

new Promise(function (resolve, reject) {
    global.require('fs').readFile(binaryPath, function (err, bytes) {
        if (err) {
            reject(err);
        }

        resolve(bytes);
    });
}).then(function (bytes) { return wasm(bytes); }).then(function () {
    wasm.set_panic_hook();
}).then(function () {
    bindings.setWasm(wasm);
});

require('cordova/exec/proxy').add('WalletPlugin', bindings.bindings);
