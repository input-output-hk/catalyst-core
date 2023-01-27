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
   * @param {string} votePlan vote plan id bytes representation
   * @param {number} proposalIndex vote's plan proposal index
   * @param {number} voteOptions number of available vote plan options, mandatory for private proposal
   * @param {string} voteEncKey committee public key in hex representation, mandatory for private proposal
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
  choice: number;
  purpose: VotingPurpose;
  expiration?: BlockDate;
  spendingCounter?: number;
  spendingCounterLane?: number;

  /**
   * Vote constructor
   *
   * @param {Proposal} proposal
   * @param {number} choice choosen vote plan option.
   * @param {VotingPurpose} purpose The voting purpose being voted on (Currently not used actually, can pass anything).
   * @param {BlockDate} expiration Deprecated field, you can pass anything.
   * @param {number} spendingCounter Deprecated field, you can pass anything.
   * @param {number} spendingCounterLane Deprecated field, you can pass anything.
   * @returns {Vote}
   */
  constructor(
    proposal: Proposal,
    choice: number,
    purpose: VotingPurpose,
    expiration?: BlockDate,
    spendingCounter?: number,
    spendingCounterLane?: number
  ): Vote;
}

/**
 * Signes provided votes and returns a completly generated transaction list
 *
 * @param {Vote[]} votes list of votes
 * @param {Settings} settings wallet Settings
 * @param {string} accountId user's account id hex representation
 * @param {string} privateKey user's private key hex representation
 * @returns {wallet_wasm.Fragment[]}
 */
function signVotes(
  votes: Vote[],
  settings: Settings,
  accountId: string,
  privateKey: string
): wallet_wasm.VoteCastTxBuilder[];
