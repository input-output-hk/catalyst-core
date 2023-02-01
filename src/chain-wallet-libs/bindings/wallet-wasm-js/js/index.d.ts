import * as wallet_wasm from "wallet-wasm-js";

export enum VotingPurpose {
  CATALYST = 0,
}

export class BlockDate {
  epoch: number;
  slot: number;

  constructor(epoch: number, slot: number): BlockDate;
}

/**
 * Wrapper over wallet-wasm-js Settings type
 */
export class Settings {
  settings: wallet_wasm.Settings;

  /**
   * Settings type constructor
   *
   * @param {string} json wallet-wasm-js Settings JSON encoded string, e.g. '{"fees":{"constant":10,"coefficient":2,"certificate":100},"discrimination":"production","block0_initial_hash":{"hash":"baf6b54817cf2a3e865f432c3922d28ac5be641e66662c66d445f141e409183e"},"block0_date":1586637936,"slot_duration":20,"time_era":{"epoch_start":0,"slot_start":0,"slots_per_epoch":180},"transaction_max_expiry_epochs":1}'
   * @returns {Settings}
   */
  constructor(json: string): Settings;

  /**
   * Returns wallet-wasm-js Settings JSON encoded string, e.g. '{"fees":{"constant":10,"coefficient":2,"certificate":100},"discrimination":"production","block0_initial_hash":{"hash":"baf6b54817cf2a3e865f432c3922d28ac5be641e66662c66d445f141e409183e"},"block0_date":1586637936,"slot_duration":20,"time_era":{"epoch_start":0,"slot_start":0,"slots_per_epoch":180},"transaction_max_expiry_epochs":1}'
   *
   * @returns {string}
   */
  toJson(): string;
}

export class Proposal {
  votePlan: string;
  proposalIndex: number;
  voteOptions?: number;
  voteEncKey?: string;

  /**
   * Proposal constructor
   *
   * @param {string} votePlan Vote plan id bytes representation
   * @param {number} proposalIndex Vote's plan proposal index
   * @param {number} voteOptions Number of available vote plan options, mandatory for private proposal
   * @param {string} voteEncKey Committee public key in hex representation, mandatory for private proposal
   * @returns {Proposal}
   */
  constructor(
    votePlan: string,
    proposalIndex: number,
    voteOptions?: number,
    voteEncKey?: string
  ): Proposal;
}

/**
 * Wrapper over wallet-wasm-js VoteCast type
 */
export class Vote {
  proposal: Proposal;
  selectedVotePlanOption: number;
  votingPurpose: VotingPurpose;
  expiration?: BlockDate;
  spendingCounter?: number;
  spendingCounterLane?: number;

  /**
   * Vote constructor
   *
   * @param {Proposal} proposal
   * @param {number} selectedVotePlanOption Selected vote plan option.
   * @param {VotingPurpose} votingPurpose The voting purpose being voted on. (Currently not used).
   * @param {BlockDate} expiration Deprecated field, you can pass anything.
   * @param {number} spendingCounter Deprecated field, you can pass anything.
   * @param {number} spendingCounterLane Deprecated field, you can pass anything.
   * @returns {Vote}
   */
  constructor(
    proposal: Proposal,
    selectedVotePlanOption: number,
    votingPurpose: VotingPurpose,
    expiration?: BlockDate,
    spendingCounter?: number,
    spendingCounterLane?: number
  ): Vote;
}

/**
 * Signes provided votes and returns a completely generated transaction list
 *
 * @param {Vote[]} votes List of votes
 * @param {Settings} settings Wallet Settings
 * @param {string} accountId User's account id hex representation
 * @param {string} privateKey Deprecated field, you can pass anything.
 * @returns {wallet_wasm.Fragment[]}
 */
function signVotes(
  votes: Vote[],
  settings: Settings,
  accountId: string,
  privateKey?: string
): wallet_wasm.VoteCastTxBuilder[];
