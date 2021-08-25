/* global BigInt */

let loaded;
const wasm = new Promise(function (resolve, reject) {
    loaded = resolve;
});

async function walletRestore (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string') {
        const mnemonics = opts[0];
        const password = '';
        try {
            const wallet = (await wasm).Wallet.recover(mnemonics, password);
            successCallback(wallet.ptr.toString());
        } catch (err) {
            errorCallback(`couldn't recover wallet ${err}`);
        }
    } else {
        errorCallback('no mnemonics provided');
    }
}

async function walletImportKeys (successCallback, errorCallback, opts) {
    try {
        const wallet = (await wasm).Wallet.import_keys(new Uint8Array(opts[0]), new Uint8Array(opts[1]));
        successCallback(wallet.ptr.toString());
    } catch (err) {
        errorCallback('error restoring from free keys ' + err);
    }
}

async function walletRetrieveFunds (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string' && opts[1] instanceof ArrayBuffer) {
        const walletPtr = opts[0];
        const block = opts[1];

        const wallet = (await wasm).Wallet.__wrap(walletPtr);

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
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        const wallet = (await wasm).Wallet.__wrap(walletPtr);

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
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        const value = opts[1];
        const counter = opts[2];
        const wallet = (await wasm).Wallet.__wrap(walletPtr);
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
    const walletPtr = opts[0];
    const settingsPtr = opts[1];
    const proposalPtr = opts[2];
    const choice = opts[3];

    const wallet = (await wasm).Wallet.__wrap(walletPtr);
    const settings = (await wasm).Settings.__wrap(settingsPtr);
    const proposal = (await wasm).Proposal.__wrap(proposalPtr);

    try {
        const tx = wallet.vote(settings, proposal, choice);
        successCallback(tx);
    } catch (err) {
        errorCallback(err);
    }
}

async function proposalNew (successCallback, errorCallback, opts) {
    const votePlanId = opts[0];
    const cordovaPayloadType = opts[1];
    const index = opts[2];
    const numChoices = opts[3];

    try {
        const id = (await wasm).VotePlanId.from_bytes(new Uint8Array(votePlanId));

        if (cordovaPayloadType === 1) {
            const options = (await wasm).Options.new_length(numChoices);
            const proposal = (await wasm).Proposal.new_public(id, index, options);
            successCallback(proposal.ptr.toString());
        } else {
            throw new Error('unrecognized payload type');
        }
    } catch (err) {
        errorCallback(err);
    }
}

async function proposalNewPublic (successCallback, errorCallback, opts) {
    const votePlanId = opts[0];
    const index = opts[1];
    const numChoices = opts[2];

    try {
        const id = (await wasm).VotePlanId.from_bytes(new Uint8Array(votePlanId));

        const options = (await wasm).Options.new_length(numChoices);
        const proposal = (await wasm).Proposal.new_public(id, index, options);
        successCallback(proposal.ptr.toString());
    } catch (err) {
        errorCallback(err);
    }
}

async function proposalNewPrivate (successCallback, errorCallback, opts) {
    const votePlanId = opts[0];
    const index = opts[1];
    const numChoices = opts[2];
    const encryptionKey = opts[3];

    const m = (await wasm);

    try {
        const id = m.VotePlanId.from_bytes(new Uint8Array(votePlanId));

        const options = m.Options.new_length(numChoices);
        const key = m.EncryptingVoteKey.from_bech32(encryptionKey);
        const proposal = m.Proposal.new_private(id, index, options, key);
        successCallback(proposal.ptr.toString());
    } catch (err) {
        errorCallback(err);
    }
}
async function walletId (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        const wallet = (await wasm).Wallet.__wrap(walletPtr);

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
    if (opts && typeof (opts[0]) === 'string' && typeof (opts[1]) === 'string') {
        const walletPtr = opts[0];
        const settingsPtr = opts[1];
        const wallet = (await wasm).Wallet.__wrap(walletPtr);
        const settings = (await wasm).Settings.__wrap(settingsPtr);

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
    if (opts && typeof (opts[0]) === 'string') {
        const conversionPtr = opts[0];
        const conversion = (await wasm).Conversion.__wrap(conversionPtr);

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
    if (opts && typeof (opts[0]) === 'string' && typeof (opts[1]) === 'number') {
        const conversionPtr = opts[0];
        const index = opts[1];
        const conversion = (await wasm).Conversion.__wrap(conversionPtr);

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
    if (opts && typeof (opts[0]) === 'string') {
        const conversionPtr = opts[0];
        const conversion = (await wasm).Conversion.__wrap(conversionPtr);

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
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        (await wasm).Wallet.__wrap(walletPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

async function settingsDelete (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string') {
        const settingsPtr = opts[0];
        (await wasm).Settings.__wrap(settingsPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

async function conversionDelete (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string') {
        const conversionPtr = opts[0];
        (await wasm).Conversion.__wrap(conversionPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

async function proposalDelete (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string') {
        const proposalPtr = opts[0];
        (await wasm).Proposal.__wrap(proposalPtr).free();
        successCallback();
    } else {
        errorCallback();
    }
}

async function walletPendingTransactions (successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string') {
        const walletPtr = opts[0];
        const wallet = (await wasm).Wallet.__wrap(walletPtr);
        try {
            const pending = wallet.pending_transactions();
            successCallback(pending);
        } catch (err) {
            errorCallback(err);
        }
    }
}

async function pendingTransactionsSize (successCallback, errorCallback, opts) {
    if (opts && Array.isArray(opts[0])) {
        const pending = opts[0];
        successCallback(pending.length);
    } else {
        errorCallback('pending transactions object is noy Array');
    }
}

async function pendingTransactionsGet (successCallback, errorCallback, opts) {
    if (opts && Array.isArray(opts[0])) {
        const pending = opts[0];
        const index = opts[1];
        successCallback(pending[index].to_bytes());
    } else {
        errorCallback('pending transactions object is noy Array');
    }
}

async function pendingTransactionsDelete (successCallback, errorCallback, opts) {
    if (opts && Array.isArray(opts[0])) {
        const pending = opts[0];
        pending.forEach(fragmentId => {
            fragmentId.free();
        });
        successCallback(pending.length);
    } else {
        errorCallback('pending transactions object is noy Array');
    }
}

async function walletConfirmTransaction (successCallback, errorCallback, opts) {
    const walletPtr = opts[0];
    const fragmentIdBytes = new Uint8Array(opts[1]);

    const wallet = (await wasm).Wallet.__wrap(walletPtr);

    try {
        const fragmentId = (await wasm).FragmentId.from_bytes(fragmentIdBytes);
        wallet.confirm_transaction(fragmentId);
        successCallback();
    } catch (err) {
        errorCallback(err);
    }
}

async function symmetricCipherDecrypt (successCallback, errorCallback, opts) {
    const password = opts[0];
    const data = opts[1];
    try {
        const decrypted = (await wasm).symmetric_decrypt(new Uint8Array(password), new Uint8Array(data));
        successCallback(decrypted);
    } catch (err) {
        errorCallback(err);
    }
}

const bindings = {
    CONVERSION_DELETE: conversionDelete,
    CONVERSION_IGNORED: conversionIgnored,
    CONVERSION_TRANSACTIONS_GET: conversionTransactionsGet,
    CONVERSION_TRANSACTIONS_SIZE: conversionTransactionsSize,
    PENDING_TRANSACTIONS_DELETE: pendingTransactionsDelete,
    PENDING_TRANSACTIONS_GET: pendingTransactionsGet,
    PENDING_TRANSACTIONS_SIZE: pendingTransactionsSize,
    PROPOSAL_DELETE: proposalDelete,
    PROPOSAL_NEW: proposalNew,
    PROPOSAL_NEW_PRIVATE: proposalNewPrivate,
    PROPOSAL_NEW_PUBLIC: proposalNewPublic,
    SETTINGS_DELETE: settingsDelete,
    SYMMETRIC_CIPHER_DECRYPT: symmetricCipherDecrypt,
    WALLET_CONFIRM_TRANSACTION: walletConfirmTransaction,
    WALLET_CONVERT: walletConvert,
    WALLET_DELETE: walletDelete,
    WALLET_ID: walletId,
    WALLET_IMPORT_KEYS: walletImportKeys,
    WALLET_PENDING_TRANSACTIONS: walletPendingTransactions,
    WALLET_RESTORE: walletRestore,
    WALLET_RETRIEVE_FUNDS: walletRetrieveFunds,
    WALLET_SET_STATE: walletSetState,
    WALLET_TOTAL_FUNDS: walletTotalFunds,
    WALLET_VOTE: walletVote
};

module.exports = { bindings: bindings, setWasm: loaded };
