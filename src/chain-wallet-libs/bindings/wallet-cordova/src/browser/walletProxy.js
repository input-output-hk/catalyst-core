const wasm = require('wallet-cordova-plugin.wasmModule');
const bindings = require('wallet-cordova-plugin.bindings');

console.log(bindings);

wasm('/wallet_js_bg.wasm').then(function () {
    wasm.set_panic_hook();
}).then(
    function () {
        bindings.setWasm(wasm);
    }
);

require('cordova/exec/proxy').add('WalletPlugin', bindings.bindings);
