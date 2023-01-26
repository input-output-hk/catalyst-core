var assert = require("assert");

const private_key =
  "c86596c2d1208885db1fe3658406aa0f7cc7b8e13c362fe46a6db277fc5064583e487588c98a6c36e2e7445c0add36f83f171cb5ccfd815509d19cd38ecb0af3";

const account_id =
  "a6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663";

const settings_json = {
  fees: {
    constant: 10,
    coefficient: 2,
    certificate: 100,
  },
  discrimination: "production",
  block0_initial_hash: {
    hash: "baf6b54817cf2a3e865f432c3922d28ac5be641e66662c66d445f141e409183e",
  },
  block0_date: 1586637936,
  slot_duration: 20,
  time_era: { epoch_start: 0, slot_start: 0, slots_per_epoch: 180 },
  transaction_max_expiry_epochs: 1,
};

describe("Inplace signing vote cast certificate tests", function () {
  it("public", async function () {
    const wallet = await import("wallet-js");

    let settings = new wallet.Settings(JSON.stringify(settings_json));
    let proposal = new wallet.Proposal(
      "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
      2,
      8
    );
    let vote = new wallet.Vote(proposal, 0);
    let tx_builders = wallet.signVotes(
      [vote],
      settings,
      account_id,
      private_key
    );
    assert(tx_builders.length == 1);
    // get fragemnts
    fragments = tx_builders.map((tx_builder) => tx_builder.finalize_tx());
  });

  it("private", async function () {
    const wallet = await import("wallet-js");

    let settings = new wallet.Settings(JSON.stringify(settings_json));
    let proposal = new wallet.Proposal(
      "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
      4,
      8,
      "bed88887abe0a84f64691fe0bdfa3daf1a6cd697a13f07ae07588910ce39c927"
    );
    let vote = new wallet.Vote(proposal, 0);
    let tx_builders = wallet.signVotes(
      [vote],
      settings,
      account_id,
      private_key
    );
    assert(tx_builders.length == 1);
    // get fragemnts
    fragments = tx_builders.map((tx_builder) => tx_builder.finalize_tx());
  });
});

describe("Postponed signing vote cast certificate tests", function () {
  it("public", async function () {
    const wallet = await import("wallet-js");

    let settings = new wallet.Settings(JSON.stringify(settings_json));
    let proposal = new wallet.Proposal(
      "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
      2,
      8
    );
    let vote = new wallet.Vote(proposal, 0);
    let tx_builders = wallet.signVotes([vote], settings, account_id);
    assert(tx_builders.length == 1);
    // get fragemnts
    fragments = tx_builders.map((tx_builder) =>
      tx_builder.sign_tx(private_key).finalize_tx()
    );
  });

  it("private", async function () {
    const wallet = await import("wallet-js");

    let settings = new wallet.Settings(JSON.stringify(settings_json));
    let proposal = new wallet.Proposal(
      "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
      4,
      8,
      "bed88887abe0a84f64691fe0bdfa3daf1a6cd697a13f07ae07588910ce39c927"
    );
    let vote = new wallet.Vote(proposal, 0);
    let tx_builders = wallet.signVotes([vote], settings, account_id);
    assert(tx_builders.length == 1);
    // get fragemnts
    fragments = tx_builders.map((tx_builder) =>
      tx_builder.sign_tx(private_key).finalize_tx()
    );
  });
});
