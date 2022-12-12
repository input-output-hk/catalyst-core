const wasm = require("wallet-wasm-js");

class BlockDate {
  constructor(epoch, slot) {
    this.epoch = epoch;
    this.slot = slot;
  }
}
module.exports.BlockDate = BlockDate;

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
  constructor(
    spending_counter,
    spending_counter_lane,
    valid_until,
    vote_plan,
    proposal_index,
    choice,
    options,
    public_key
  ) {
    this.spending_counter = spending_counter;
    this.spending_counter_lane = spending_counter_lane;
    this.valid_until = valid_until;
    if (options != undefined && public_key != undefined) {
      let payload = wasm.Payload.new_private(
        vote_plan,
        options,
        choice,
        public_key
      );
      this.vote_cast = wasm.VoteCast.new(vote_plan, proposal_index, payload);
    } else {
      let payload = wasm.Payload.new_public(choice);
      this.vote_cast = wasm.VoteCast.new(vote_plan, proposal_index, payload);
    }
  }
}
module.exports.Vote = Vote;

class Proposal {
  constructor(
    vote_plan_bytes,
    options_count,
    vote_public,
    proposal_index,
    committee_public_key
  ) {
    this.vote_plan = wasm.VotePlanId.from_bytes(vote_plan_bytes);
    this.options_count = options_count;
    this.vote_public = vote_public;
    this.proposal_index = proposal_index;
    this.committee_public_key = committee_public_key;
  }

  vote(spending_counter, spending_counter_lane, valid_until, choice) {
    if (this.vote_public) {
      return new Vote(
        spending_counter,
        spending_counter_lane,
        valid_until,
        this.vote_plan,
        this.proposal_index,
        choice
      );
    } else {
      return new Vote(
        spending_counter,
        spending_counter_lane,
        valid_until,
        this.vote_plan,
        this.proposal_index,
        choice,
        this.options_count,
        this.committee_public_key
      );
    }
  }
}
module.exports.Proposal = Proposal;

function signVotes(votes, settings, private_key) {
  let fragments = [];
  for (let i = 0; i < votes.length; i++) {
    let builder = wasm.VoteCastTxBuilder.new(
      settings.settings,
      votes[i].valid_until.epoch,
      votes[i].valid_until.slot,
      votes[i].vote_cast
    );
    let fragment = builder
      .build_tx(
        private_key,
        votes[i].spending_counter,
        votes[i].spending_counter_lane
      )
      .finalize_tx();
    fragments.push(fragment);
  }
  return fragments;
}
module.exports.signVotes = signVotes;
