# README

# Usage

## Examples

### Wallet conversion

Minimal wallet conversion example in plain javascript

``` js
var lib = cordova.require('wallet-cordova-plugin.wallet');
// or 
// var wallet = window.wallet;

function handleError(error) {
    console.error('error: ' + error);
}

function getTransactionsFromConversion(conversion) {
    lib.conversionTransactionsSize(conversion, function(size) {
        const transactions = [];
        for (var i = 0; i < size; i++) {
            lib.conversionTransactionsGet(conversion, i, function(tx) {
                transactions.push(tx);

                if (transactions.length == size) {
                    // do something with the transactions, for example, POST to jormungandr
                }
            }, handleError)
        }
    }, handleError)
}

lib.walletRestore(mnemonics, function(wallet) {
    lib.walletRetrieveFunds(wallet, BLOCK0, function(settings) {
        lib.walletTotalFunds(wallet, function(retrievedFunds) {
            console.log('retrieved: ' + retrievedFunds + ' funds from block0');
            lib.walletConvert(wallet, settings, getTransactionsFromConversion, handleError)
        }, handleError);
    }, handleError)
}, handleError);
```

# Electron quirks

At the moment, the plugin requires that node integrations are enabled in the app.
