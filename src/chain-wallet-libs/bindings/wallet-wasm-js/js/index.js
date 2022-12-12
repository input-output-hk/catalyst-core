const wasm = require("wallet-wasm-js");

module.exports.SpendingCounter = wasm.SpendingCounter;
module.exports.SpendingCounters = wasm.SpendingCounters;
module.exports.VotePlanId = wasm.VotePlanId;
module.exports.Payload = wasm.Payload;
module.exports.VoteCast = wasm.VoteCast;
module.exports.BlockDate = wasm.BlockDate;
module.exports.Certificate = wasm.Certificate;

class Settings {
  constructor(json) {
    this.settings = wasm.Settings.from_json(json);
  }

  to_json() {
    return this.settings.to_json();
  }
}
module.exports.Settings = Settings;

class Vote {
  static public(vote_plan_bytes, proposal_index, choice) {
    let res = new Vote();
    let vote_plan = wasm.VotePlanId.from_bytes(vote_plan_bytes);
    let payload = wasm.Payload.new_public(choice);
    let vote_cast = wasm.VoteCast.new(vote_plan, proposal_index, payload);
    res.vote_cast = vote_cast;
    return res;
  }

  static private(vote_plan_bytes, proposal_index, options, choice, public_key) {
    let res = new Vote();
    let vote_plan = wasm.VotePlanId.from_bytes(vote_plan_bytes);
    let payload = wasm.Payload.new_private(
      vote_plan,
      options,
      choice,
      public_key
    );
    vote_plan = wasm.VotePlanId.from_bytes(vote_plan_bytes);
    let vote_cast = wasm.VoteCast.new(vote_plan, proposal_index, payload);
    res.vote_cast = vote_cast;
    return res;
  }
}
module.exports.Vote = Vote;

const MAX_LANES = 8;

class Wallet {
  constructor(private_key, init_value) {
    this.wallet = wasm.Wallet.import_key(private_key, init_value);

    let spending_counters = wasm.SpendingCounters.new();
    for (let lane = 0; lane < MAX_LANES; lane++) {
      let spending_counter = wasm.SpendingCounter.new(lane, 1);
      spending_counters.add(spending_counter);
    }

    this.wallet.set_state(init_value, spending_counters);
  }

  total_value() {
    return this.wallet.total_value();
  }
}
module.exports.Wallet = Wallet;

function signVotes(wallet, votes, settings, valid_until, lane) {
  let fragments = [];
  for (let i = 0; i < votes.length; i++) {
    let certificate = wasm.Certificate.vote_cast(votes[i].vote_cast);
    let fragment = wallet.wallet.sign_transaction(
      settings.settings,
      valid_until,
      lane,
      certificate
    );
    wallet.wallet.confirm_transaction(fragment.id());
    fragments.push(fragment);
  }
  return fragments;
}
module.exports.signVotes = signVotes;

