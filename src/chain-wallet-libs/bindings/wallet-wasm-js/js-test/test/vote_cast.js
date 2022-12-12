var assert = require("assert");

const private_key = Buffer.from(
  "c86596c2d1208885db1fe3658406aa0f7cc7b8e13c362fe46a6db277fc5064583e487588c98a6c36e2e7445c0add36f83f171cb5ccfd815509d19cd38ecb0af3",
  "hex"
);

const settings_json =
  '{"fees":{"constant":10,"coefficient":2,"certificate":100},"discrimination":"production","block0_initial_hash":{"hash":"baf6b54817cf2a3e865f432c3922d28ac5be641e66662c66d445f141e409183e"},"block0_date":1586637936,"slot_duration":20,"time_era":{"epoch_start":0,"slot_start":0,"slots_per_epoch":180},"transaction_max_expiry_epochs":1}';

describe("vote cast certificate tests", function () {
  it("public", async function () {
    const wasm_wallet = await import("wallet-js");

    let settings = new wasm_wallet.Settings(settings_json);
    let vote = wasm_wallet.Vote.public(
      wasm_wallet.SpendingCounter.new(1, 1),
      Buffer.from(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        "hex"
      ),
      8,
      0
    );

    let block_date = wasm_wallet.BlockDate.new(0, 1);
    let fragments = wasm_wallet.signVotes(
      [vote],
      private_key,
      settings,
      block_date
    );
    assert(fragments.length == 1);
  });

  it("private", async function () {
    const wasm_wallet = await import("wallet-js");

    let settings = new wasm_wallet.Settings(settings_json);
    let vote = wasm_wallet.Vote.private(
      wasm_wallet.SpendingCounter.new(1, 1),
      Buffer.from(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        "hex"
      ),
      8,
      4,
      0,
      Buffer.from(
        "bed88887abe0a84f64691fe0bdfa3daf1a6cd697a13f07ae07588910ce39c927",
        "hex"
      )
    );

    let block_date = wasm_wallet.BlockDate.new(0, 1);
    let fragments = wasm_wallet.signVotes(
      [vote],
      private_key,
      settings,
      block_date
    );
    assert(fragments.length == 1);
  });
});
