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
    spendingCounter,
    spendingCounterLane,
    validUntil,
    votePlan,
    proposalIndex,
    choice,
    options,
    publicKey
  ) {
    this.spendingCounter = spendingCounter;
    this.spendingCounterLane = spendingCounterLane;
    this.validUntil = validUntil;
    if (options != undefined && publicKey != undefined) {
      let payload = wasm.Payload.new_private(
        wasm.VotePlanId.from_bytes(votePlan),
        options,
        choice,
        publicKey
      );
      this.voteCast = wasm.VoteCast.new(
        wasm.VotePlanId.from_bytes(votePlan),
        proposalIndex,
        payload
      );
    } else {
      let payload = wasm.Payload.new_public(choice);
      this.voteCast = wasm.VoteCast.new(
        wasm.VotePlanId.from_bytes(votePlan),
        proposalIndex,
        payload
      );
    }
  }
}
module.exports.Vote = Vote;

class Proposal {
  constructor(
    votePlan,
    voteOptions,
    votePublic,
    proposalIndex,
    committee_public_key
  ) {
    this.votePlan = votePlan;
    this.voteOptions = voteOptions;
    this.votePublic = votePublic;
    this.proposalIndex = proposalIndex;
    this.committee_public_key = committee_public_key;
  }

  vote(spendingCounter, spendingCounterLane, validUntil, choice) {
    if (this.votePublic) {
      return new Vote(
        spendingCounter,
        spendingCounterLane,
        validUntil,
        this.votePlan,
        this.proposalIndex,
        choice
      );
    } else {
      return new Vote(
        spendingCounter,
        spendingCounterLane,
        validUntil,
        this.votePlan,
        this.proposalIndex,
        choice,
        this.voteOptions,
        this.committee_public_key
      );
    }
  }
}
module.exports.Proposal = Proposal;

function signVotes(votes, settings, privateKey) {
  let fragments = [];
  for (let i = 0; i < votes.length; i++) {
    let builder = wasm.VoteCastTxBuilder.new(
      settings.settings,
      votes[i].validUntil.epoch,
      votes[i].validUntil.slot,
      votes[i].voteCast
    );
    let fragment = builder
      .build_tx(
        privateKey,
        votes[i].spendingCounter,
        votes[i].spendingCounterLane
      )
      .finalize_tx();
    fragments.push(fragment);
  }
  return fragments;
}
module.exports.signVotes = signVotes;
