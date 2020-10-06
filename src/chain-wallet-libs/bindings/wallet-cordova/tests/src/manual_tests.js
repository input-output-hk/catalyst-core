const primitives = require('wallet-cordova-plugin.wallet');

const { hex, hexStringToBytes } = require('./utils.js');

// TODO: untangle this nesting hell. I still don't know if I can use promises/async here
function restoreManualInputWallet(mnemonics, hexBlock, callBack) {
    window.wallet.walletRestore(mnemonics, wallet => {
        window.wallet.walletRetrieveFunds(wallet, hexStringToBytes(hexBlock), settings => {
            window.wallet.walletTotalFunds(wallet, retrievedFunds => {
                window.wallet.settingsDelete(settings, () => {
                    window.wallet.walletDelete(wallet, () => {
                        callBack(undefined, retrievedFunds);
                    }, err => { callBack(new Error(`couldn't delete wallet ${err}`)); });
                }, err => { callBack(new Error(`couldn't delete settings ${err}`)); });
            }, err => { callBack(new Error(`couldn't get total funds ${err}`)); });
        }, err => {
            callBack(new Error(`could not retrieve funds ${err}`));
        });
    }, err => {
        callBack(new Error(`could not create wallet ${err}`));
    });
}

function getAccountId(mnemonics, callBack) {
    primitives.walletRestore(mnemonics, wallet => {
        primitives.walletId(wallet, function (id) {
            callBack(undefined, hex(id));
        }, function (err) {
            callBack(new Error(`could not get account id ${err}`));
        });
    }, err => {
        callBack(new Error(`could not create wallet ${err}`));
    });
}

module.exports = function (contentEl, createActionButton) {
    var logMessage = function (message, color) {
        var log = document.getElementById('info');
        var logLine = document.createElement('div');
        if (color) {
            logLine.style.color = color;
        }
        logLine.innerHTML = message;
        log.appendChild(logLine);
    };

    var clearLog = function () {
        var log = document.getElementById('info');
        log.innerHTML = '';
    };

    const form =
        '<div> <label> mnemonics </label> <textarea id="mnemonics" rows="1"></textarea> </div>' +
        '<div> <label> block(hex) </label> <textarea id="block" rows="1"></textarea> </div>' +
        '<div id="get_funds"> </div>' +
        '<div id="account"> </div>';

    contentEl.innerHTML = '<div id="info"></div>' + form;

    createActionButton(
        'get funds',
        function () {
            clearLog();
            const mnemonics = document.getElementById('mnemonics').value;
            const block = document.getElementById('block').value;
            restoreManualInputWallet(mnemonics, block, (error, value) => {
                if (error) {
                    logMessage(`Error: ${error}`, null);
                } else {
                    logMessage(`Funds: ${value}`, null);
                }
            });
        },
        'get_funds'
    );

    createActionButton(
        'get account id',
        function () {
            clearLog();
            const mnemonics = document.getElementById('mnemonics').value;
            getAccountId(mnemonics, (error, value) => {
                if (error) {
                    logMessage(`Error: ${error}`, null);
                } else {
                    logMessage(`account id: ${value}`, null);
                }
            });
        },
        'account'
    );
};
