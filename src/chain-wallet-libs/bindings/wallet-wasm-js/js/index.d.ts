import { BlockDate, Fragment } from "wallet_js";

export class Settings {
    constructor(json: string): Settings;

    to_json(): string;
}

export class Vote {

    static public(vote_plan_bytes: Uint8Array, proposal_index: number, choice: number): Vote;

    static private(vote_plan_bytes: Uint8Array, proposal_index: number, options: number, choice: number, public_key: Uint8Array): Vote;
}

export class Wallet {
    constructor(private_key: Uint8Array, init_value: bigint): Wallet;

    signVotes(votes: [Vote], settings: Settings, valid_until: BlockDate, lane: number): Fragment;

    total_value(): bigint;
}