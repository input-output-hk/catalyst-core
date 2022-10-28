// const fs = require('fs');
// const source = fs.readFileSync('wallet_js_bg.wasm');

let source = fetch("file:///Users/alexeypoghilenkov/Work/catalyst-core/src/chain-wallet-libs/bindings/wallet-js/js-tests/wallet_js_bg.wasm");

const importObject = { imports: { imported_func: (arg) => console.log(arg) } };

WebAssembly.instantiateStreaming(source, importObject).then(
    (results) => {
        
    }
)