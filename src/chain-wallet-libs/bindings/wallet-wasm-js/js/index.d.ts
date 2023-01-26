import * as wallet_wasm from "wallet-wasm-js";

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
   * @param {number} voteOptions number of available vote plan options
   * @param {number} proposalIndex vote's plan proposal index, mandatory for private proposal
   * @param {string} voteEncKey committee public key in hex representation, mandatory for private proposal
   * @returns {Proposal}
   */
  constructor(
    votePlan: string,
    voteOptions: number,
    proposalIndex?: number,
    voteEncKey?: string
  ): Proposal;
}

/**
 * Wrapper over wallet-wasm-js VoteCast type
 */
export class Vote {
  proposal: Proposal;
  choice: number;

  /**
   * Vote constructor
   *
   * @param {Proposal} proposal
   * @param {number} choice choosen vote plan option
   * @returns {Vote}
   */
  constructor(proposal: Proposal, choice: number): Vote;
}

/**
 * Signes provided votes and returns a completly generated transaction list
 *
 * @param {Vote[]} votes list of votes
 * @param {Settings} settings wallet Settings
 * @param {string} privateKey user private key hex representation
 * @returns {wallet_wasm.Fragment[]}
 */
function signVotes(
  votes: Vote[],
  settings: Settings,
  privateKey: string
): wallet_wasm.Fragment[];
