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

class Proposal {
  constructor(votePlan, voteOptions, proposalIndex, voteEncKey) {
    this.votePlan = votePlan;
    this.voteOptions = voteOptions;
    this.proposalIndex = proposalIndex;
    this.voteEncKey = voteEncKey;
  }
}
module.exports.Proposal = Proposal;

class Vote {
  constructor(
    proposal,
    choice,
    expiration,
    spendingCounter,
    spendingCounterLane
  ) {
    this.proposal = proposal;
    this.choice = choice;
    this.expiration = expiration;
    this.spendingCounter = spendingCounter;
    this.spendingCounterLane = spendingCounterLane;
  }
}
module.exports.Vote = Vote;

function signVotes(votes, settings, accountId, privateKey) {
  let tx_builders = [];
  for (let i = 0; i < votes.length; i++) {
    let vote = votes[i];
    let voteCast;
    if (
      vote.proposal.options != undefined &&
      vote.proposal.voteEncKey != undefined
    ) {
      let payload = wasm.Payload.new_private(
        wasm.VotePlanId.from_hex(vote.proposal.votePlan),
        vote.proposal.options,
        vote.choice,
        vote.proposal.voteEncKey
      );
      voteCast = wasm.VoteCast.new(
        wasm.VotePlanId.from_hex(vote.proposal.votePlan),
        vote.proposal.proposalIndex,
        payload
      );
    } else {
      let payload = wasm.Payload.new_public(vote.choice);
      voteCast = wasm.VoteCast.new(
        wasm.VotePlanId.from_hex(vote.proposal.votePlan),
        vote.proposal.proposalIndex,
        payload
      );
    }

    let builder = wasm.VoteCastTxBuilder.new(
      settings.settings,
      vote.expiration.epoch,
      vote.expiration.slot,
      voteCast
    );
    let tx_builder = builder.build_tx(
      accountId,
      vote.spendingCounter,
      vote.spendingCounterLane
    );
    if (privateKey != undefined) {
      tx_builder = tx_builder.sign_tx(privateKey);
    }
    tx_builders.push(tx_builder);
  }
  return tx_builders;
}
module.exports.signVotes = signVotes;
