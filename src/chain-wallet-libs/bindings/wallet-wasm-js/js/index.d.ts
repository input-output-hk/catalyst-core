import * as wallet_wasm from "wallet-wasm-js";

export class Settings {
  settings: wallet_wasm.Settings;

  constructor(json: string): Settings;

  to_json(): string;
}

export class Vote {
  vote_cast: wallet_wasm.VoteCast;

  static public(
    vote_plan_bytes: Uint8Array,
    proposal_index: number,
    choice: number
  ): Vote;

  static private(
    vote_plan_bytes: Uint8Array,
    proposal_index: number,
    options: number,
    choice: number,
    public_key: Uint8Array
  ): Vote;
}

export class Wallet {
  wallet: wallet_wasm.Wallet;

  constructor(private_key: Uint8Array, init_value: bigint): Wallet;

  signVotes(
    votes: [Vote],
    settings: Settings,
    valid_until: wallet_wasm.BlockDate,
    lane: number
  ): wallet_wasm.Fragment;

  total_value(): bigint;
}
