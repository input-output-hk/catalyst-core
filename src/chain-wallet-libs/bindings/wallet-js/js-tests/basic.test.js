

test('load wallet module', () => {
    WebAssembly.instantiateStreaming(fetch("wallet_js_bg.wasm"), importObject).then(
        (results) => {

        }
    )
});