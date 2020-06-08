/* global BigInt */
const wasm = require('wallet-cordova-plugin.wasmModule');

async function walletRestore (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const mnemonics = opts[0];
        const password = '';
        try {
            const wallet = wasm.Wallet.recover(mnemonics, password);
            successCallback(wallet.ptr.toString());
        } catch (err) {
            errorCallback(`couldn't recover wallet ${err}`);
        }
    } else {
        errorCallback('no mnemonics provided');
    }
}

async function walletRetrieveFunds (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string' && opts[1] instanceof ArrayBuffer) {
        const walletPtr = opts[0];
        const block = opts[1];

        const wallet = wasm.Wallet.__wrap(walletPtr);

        try {
            const settings = wallet.retrieve_funds(new Uint8Array(block));
            successCallback(settings.ptr.toString());
        } catch (err) {
            errorCallback(`couldn't retrieve funds ${err}`);
        }
    } else {
        errorCallback('missing walletPtr or block');
    }
}

async function walletTotalFunds (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        const wallet = wasm.Wallet.__wrap(walletPtr);

        try {
            successCallback(wallet.total_value().toString());
        } catch (err) {
            errorCallback(`couldn't get funds ${err}`);
        }
    } else {
        errorCallback('no pointer');
    }
}

async function walletSetState (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        const value = opts[1];
        const counter = opts[2];
        const wallet = wasm.Wallet.__wrap(walletPtr);
        try {
            wallet.set_state(BigInt(value), counter);
            successCallback();
        } catch (err) {
            errorCallback(err);
        }
    } else {
        errorCallback('invalid wallet pointer');
    }
}

async function walletVote (successCallback, errorCallback, opts) {
    await loaded;
    const walletPtr = opts[0];
    const settingsPtr = opts[1];
    const proposalPtr = opts[2];
    const choice = opts[3];

    const wallet = wasm.Wallet.__wrap(walletPtr);
    const settings = wasm.Settings.__wrap(settingsPtr);
    const proposal = wasm.Proposal.__wrap(proposalPtr);

    try {
        const tx = wallet.vote(settings, proposal, choice);
        successCallback(tx);
    } catch (err) {
        errorCallback(err);
    }
}

async function proposalNew (successCallback, errorCallback, opts) {
    await loaded;

    const votePlanId = opts[0];
    const cordovaPayloadType = opts[1];
    const index = opts[2];
    const numChoices = opts[3];

    try {
        const id = wasm.VotePlanId.new_from_bytes(votePlanId);
        let payloadType;

        if (cordovaPayloadType === 1) {
            payloadType = wasm.PayloadType.Public;
        } else {
            throw new Error('unrecognized payload type');
        }

        const options = wasm.Options.new_length(numChoices);
        const proposal = wasm.Proposal.new(id, payloadType, index, options);
        successCallback(proposal.ptr.toString());
    } catch (err) {
        errorCallback(err);
    }
}

async function walletId (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        const wallet = wasm.Wallet.__wrap(walletPtr);

        try {
            successCallback(wallet.id());
        } catch (err) {
            errorCallback(`couldn't get funds ${err}`);
        }
    } else {
        errorCallback('no pointer');
    }
}

async function walletConvert (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string' && typeof (opts[1]) === 'string') {
        const walletPtr = opts[0];
        const settingsPtr = opts[1];
        const wallet = wasm.Wallet.__wrap(walletPtr);
        const settings = wasm.Settings.__wrap(settingsPtr);

        try {
            successCallback(wallet.convert(settings).ptr.toString());
        } catch (err) {
            errorCallback(`couldn't get funds ${err}`);
        }
    } else {
        errorCallback('no pointer');
    }
}

async function conversionTransactionsSize (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const conversionPtr = opts[0];
        const conversion = wasm.Conversion.__wrap(conversionPtr);

        try {
            successCallback(conversion.transactions_len());
        } catch (err) {
            errorCallback(`couldn't get transactions size: ${err}`);
        }
    } else {
        errorCallback('no pointer');
    }
}

async function conversionTransactionsGet (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string' && typeof (opts[1]) === 'number') {
        const conversionPtr = opts[0];
        const index = opts[1];
        const conversion = wasm.Conversion.__wrap(conversionPtr);

        try {
            successCallback(conversion.transactions_get(index));
        } catch (err) {
            errorCallback(`couldn't get transaction at index: ${index} - error: ${err}`);
        }
    } else {
        errorCallback('no pointer');
    }
}

async function conversionIgnored (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const conversionPtr = opts[0];
        const conversion = wasm.Conversion.__wrap(conversionPtr);

        try {
            const ignored = conversion.num_ignored();
            const value = conversion.total_value_ignored();
            successCallback({ ignored, value });
        } catch (err) {
            errorCallback(err);
        }
    } else {
        errorCallback('invalid or missing conversion pointer');
    }
}

async function walletDelete (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        wasm.Wallet.__wrap(walletPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

async function settingsDelete (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const settingsPtr = opts[0];
        wasm.Settings.__wrap(settingsPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

async function conversionDelete (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const conversionPtr = opts[0];
        wasm.Conversion.__wrap(conversionPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

async function proposalDelete (successCallback, errorCallback, opts) {
    await loaded;
    if (opts && typeof (opts[0]) === 'string') {
        const conversionPtr = opts[0];
        wasm.Conversion.__wrap(conversionPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

const bindings = {
    WALLET_RESTORE: walletRestore,
    WALLET_RETRIEVE_FUNDS: walletRetrieveFunds,
    WALLET_TOTAL_FUNDS: walletTotalFunds,
    WALLET_ID: walletId,
    WALLET_SET_STATE: walletSetState,
    WALLET_VOTE: walletVote,
    PROPOSAL_NEW: proposalNew,
    WALLET_CONVERT: walletConvert,
    CONVERSION_TRANSACTIONS_SIZE: conversionTransactionsSize,
    CONVERSION_TRANSACTIONS_GET: conversionTransactionsGet,
    CONVERSION_IGNORED: conversionIgnored,
    WALLET_DELETE: walletDelete,
    SETTINGS_DELETE: settingsDelete,
    CONVERSION_DELETE: conversionDelete,
    PROPOSAL_DELETE: proposalDelete
};

require('cordova/exec/proxy').add('WalletPlugin', bindings);

// this is in done in order to make it work regardless of where the html file is located
const appPath = global.require('electron').remote.app.getAppPath();
const binaryPath = global.require('path').join(appPath, 'wallet_js_bg.wasm');

const loaded = new Promise(function (resolve, reject) {
    global.require('fs').readFile(binaryPath, function (err, bytes) {
        if (err) {
            reject(err);
        }

        resolve(bytes);
    });
}).then(function (bytes) { return wasm(bytes); }).then(function () {
    wasm.set_panic_hook();
});
