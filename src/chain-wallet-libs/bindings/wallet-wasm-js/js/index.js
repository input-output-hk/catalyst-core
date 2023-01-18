import wasm from "wallet-wasm-js";

export class BlockDate {
  constructor(epoch, slot) {
    this.epoch = epoch;
    this.slot = slot;
  }
}

export class Settings {
  constructor(json) {
    this.settings = wasm.Settings.from_json(json);
  }

  to_json() {
    return this.settings.to_json();
  }
}

export class Proposal {
  constructor(votePlan, voteOptions, proposalIndex, voteEncKey) {
    this.votePlan = votePlan;
    this.voteOptions = voteOptions;
    this.proposalIndex = proposalIndex;
    this.voteEncKey = voteEncKey;
  }
}

export class Vote {
  constructor(
    proposal,
    choice,
    expiration,
    spendingCounter,
    spendingCounterLane
  ) {
    this.proposal = proposal;
    this.choice = choice;
    this.spendingCounter = spendingCounter;
    this.spendingCounterLane = spendingCounterLane;
  }
}

export function signVotes(votes, settings, accountId, privateKey) {
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
