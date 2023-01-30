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
  constructor(votePlan, proposalIndex, voteOptions, voteEncKey) {
    this.votePlan = votePlan;
    this.proposalIndex = proposalIndex;
    this.voteOptions = voteOptions;
    this.voteEncKey = voteEncKey;
  }
}

export class Vote {
  constructor(proposal, choice, purpose) {
    this.proposal = proposal;
    this.choice = choice;
    this.purpose = purpose;
  }
}

export function signVotes(votes, settings, accountId, privateKey) {
  let tx_builders = [];
  for (let i = 0; i < votes.length; i++) {
    let vote = votes[i];
    let payload;
    if (
      vote.proposal.voteOptions != undefined &&
      vote.proposal.voteEncKey != undefined
    ) {
      payload = wasm.Payload.new_private(
        wasm.VotePlanId.from_hex(vote.proposal.votePlan),
        vote.proposal.voteOptions,
        vote.choice,
        wasm.ElectionPublicKey.from_hex(vote.proposal.voteEncKey)
      );
    } else {
      payload = wasm.Payload.new_public(vote.choice);
    }
    let voteCast = wasm.VoteCast.new(
      wasm.VotePlanId.from_hex(vote.proposal.votePlan),
      vote.proposal.proposalIndex,
      payload
    );

    let builder = wasm.VoteCastTxBuilder.new(settings.settings, voteCast);
    let tx_builder = builder.build_tx(accountId);
    tx_builders.push(tx_builder);
  }
  return tx_builders;
}
