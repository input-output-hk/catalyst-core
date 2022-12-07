import * as wallet_wasm from "wallet-wasm-js";

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

/**
 * Wrapper over wallet-wasm-js VoteCast type
 */
export class Vote {
  voteCast: wallet_wasm.VoteCast;

  /**
   * Constructs public wallet-wasm-js VoteCast vote
   *
   * @param {Uint8Array} vote_plan_bytes vote plan id bytes representation
   * @param {number} proposal_index vote's plan proposal index
   * @param {number} choice choosen vote plan option
   * @returns {Vote}
   */
  static public(
    vote_plan_bytes: Uint8Array,
    proposal_index: number,
    choice: number
  ): Vote;

  /**
   * Constructs public wallet-wasm-js VoteCast vote
   *
   * @param {Uint8Array} vote_plan_bytes vote plan id bytes representation
   * @param {number} proposal_index vote's plan proposal index
   * @param {number} options number of available vote plan options
   * @param {number} choice choosen vote plan option
   * @param {Uint8Array} public_key committee public key bytes representation
   * @returns {Vote}
   */
  static private(
    vote_plan_bytes: Uint8Array,
    proposal_index: number,
    options: number,
    choice: number,
    public_key: Uint8Array
  ): Vote;
}

/**
 * Wrapper over wallet-wasm-js Wallet type
 */
export class Wallet {
  wallet: wallet_wasm.Wallet;

  /**
   * Wallet type constructor
   *
   * @param {Uint8Array} private_key user private key bytes representation
   * @param {bigint} init_value wallet initial balance value
   * @returns {Wallet}
   */
  constructor(private_key: Uint8Array, init_value: bigint): Wallet;

  /**
   * Signes provided votes and returns a completly generated transaction list
   *
   * @param {Vote[]} votes list of votes
   * @param {Settings} settings wallet Settings
   * @param {wallet_wasm.BlockDate} valid_until 
   * @param {number} lane
   * @returns {wallet_wasm.Fragment[]}
   */
  signVotes(
    votes: Vote[],
    settings: Settings,
    valid_until: wallet_wasm.BlockDate,
    lane: number
  ): wallet_wasm.Fragment[];

  /**
   * Returns current balance of the wallet
   *
   * @returns {bigint}
   */
  totalValue(): bigint;
}
