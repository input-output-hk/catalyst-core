const wasm = require("wallet-js");

module.exports.Wallet = wasm.Wallet;
module.exports.Settings = wasm.Settings;
module.exports.SpendingCounter = wasm.SpendingCounter;
module.exports.SpendingCounters = wasm.SpendingCounters;
module.exports.VotePlanId = wasm.VotePlanId;
module.exports.Payload = wasm.Payload;
module.exports.VoteCast = wasm.VoteCast;
module.exports.BlockDate = wasm.BlockDate;
module.exports.Certificate = wasm.Certificate;


class Wallet {
    constructor(private_key) {
        this.wallet = wasm.Wallet.import_key(private_key);
    }
}
