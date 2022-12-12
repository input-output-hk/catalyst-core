const wasm = require("wallet-wasm-js");

module.exports.SpendingCounter = wasm.SpendingCounter;
module.exports.SpendingCounters = wasm.SpendingCounters;
module.exports.VotePlanId = wasm.VotePlanId;
module.exports.VoteCast = wasm.VoteCast;
module.exports.BlockDate = wasm.BlockDate;

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
  static public(spending_counter, vote_plan_bytes, proposal_index, choice) {
    let res = new Vote();
    let vote_plan = wasm.VotePlanId.from_bytes(vote_plan_bytes);
    let payload = wasm.Payload.new_public(choice);
    let vote_cast = wasm.VoteCast.new(vote_plan, proposal_index, payload);
    res.vote_cast = vote_cast;
    res.spending_counter = spending_counter;
    return res;
  }

  static private(
    spending_counter,
    vote_plan_bytes,
    proposal_index,
    options,
    choice,
    public_key
  ) {
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
    res.spending_counter = spending_counter;
    return res;
  }
}
module.exports.Vote = Vote;

function signVotes(votes, settings, valid_until, private_key) {
  let fragments = [];
  for (let i = 0; i < votes.length; i++) {
    let builder = wasm.VoteCastTxBuilder.new(
      settings.settings,
      valid_until,
      votes[i].vote_cast
    );
    let fragment = builder
      .build_tx(private_key, votes[i].spending_counter)
      .finalize_tx();
    fragments.push(fragment);
  }
  return fragments;
}
module.exports.signVotes = signVotes;
