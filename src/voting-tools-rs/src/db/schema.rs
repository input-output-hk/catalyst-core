table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    ada_pots (id) {
        id -> Int8,
        slot_no -> Int8,
        epoch_no -> Int4,
        treasury -> Numeric,
        reserves -> Numeric,
        rewards -> Numeric,
        utxo -> Numeric,
        deposits -> Numeric,
        fees -> Numeric,
        block_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    block (id) {
        id -> Int8,
        hash -> Bytea,
        epoch_no -> Nullable<Int4>,
        slot_no -> Nullable<Int8>,
        epoch_slot_no -> Nullable<Int4>,
        block_no -> Nullable<Int4>,
        previous_id -> Nullable<Int8>,
        slot_leader_id -> Int8,
        size -> Int4,
        time -> Timestamp,
        tx_count -> Int8,
        proto_major -> Int4,
        proto_minor -> Int4,
        vrf_key -> Nullable<Varchar>,
        op_cert -> Nullable<Bytea>,
        op_cert_counter -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    collateral_tx_in (id) {
        id -> Int8,
        tx_in_id -> Int8,
        tx_out_id -> Int8,
        tx_out_index -> Int2,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    collateral_tx_out (id) {
        id -> Int8,
        tx_id -> Int8,
        index -> Int2,
        address -> Varchar,
        address_raw -> Bytea,
        address_has_script -> Bool,
        payment_cred -> Nullable<Bytea>,
        stake_address_id -> Nullable<Int8>,
        value -> Numeric,
        data_hash -> Nullable<Bytea>,
        multi_assets_descr -> Varchar,
        inline_datum_id -> Nullable<Int8>,
        reference_script_id -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    cost_model (id) {
        id -> Int8,
        costs -> Jsonb,
        block_id -> Int8,
        hash -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    datum (id) {
        id -> Int8,
        hash -> Bytea,
        tx_id -> Int8,
        value -> Nullable<Jsonb>,
        bytes -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    delegation (id) {
        id -> Int8,
        addr_id -> Int8,
        cert_index -> Int4,
        pool_hash_id -> Int8,
        active_epoch_no -> Int8,
        tx_id -> Int8,
        slot_no -> Int8,
        redeemer_id -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    delisted_pool (id) {
        id -> Int8,
        hash_raw -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    epoch (id) {
        id -> Int8,
        out_sum -> Numeric,
        fees -> Numeric,
        tx_count -> Int4,
        blk_count -> Int4,
        no -> Int4,
        start_time -> Timestamp,
        end_time -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    epoch_param (id) {
        id -> Int8,
        epoch_no -> Int4,
        min_fee_a -> Int4,
        min_fee_b -> Int4,
        max_block_size -> Int4,
        max_tx_size -> Int4,
        max_bh_size -> Int4,
        key_deposit -> Numeric,
        pool_deposit -> Numeric,
        max_epoch -> Int4,
        optimal_pool_count -> Int4,
        influence -> Float8,
        monetary_expand_rate -> Float8,
        treasury_growth_rate -> Float8,
        decentralisation -> Float8,
        protocol_major -> Int4,
        protocol_minor -> Int4,
        min_utxo_value -> Numeric,
        min_pool_cost -> Numeric,
        nonce -> Nullable<Bytea>,
        cost_model_id -> Nullable<Int8>,
        price_mem -> Nullable<Float8>,
        price_step -> Nullable<Float8>,
        max_tx_ex_mem -> Nullable<Numeric>,
        max_tx_ex_steps -> Nullable<Numeric>,
        max_block_ex_mem -> Nullable<Numeric>,
        max_block_ex_steps -> Nullable<Numeric>,
        max_val_size -> Nullable<Numeric>,
        collateral_percent -> Nullable<Int4>,
        max_collateral_inputs -> Nullable<Int4>,
        block_id -> Int8,
        extra_entropy -> Nullable<Bytea>,
        coins_per_utxo_size -> Nullable<Numeric>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    epoch_stake (id) {
        id -> Int8,
        addr_id -> Int8,
        pool_id -> Int8,
        amount -> Numeric,
        epoch_no -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    epoch_sync_time (id) {
        id -> Int8,
        no -> Int8,
        seconds -> Int8,
        state -> Syncstatetype,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    extra_key_witness (id) {
        id -> Int8,
        hash -> Bytea,
        tx_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    ma_tx_mint (id) {
        id -> Int8,
        quantity -> Numeric,
        tx_id -> Int8,
        ident -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    ma_tx_out (id) {
        id -> Int8,
        quantity -> Numeric,
        tx_out_id -> Int8,
        ident -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    meta (id) {
        id -> Int8,
        start_time -> Timestamp,
        network_name -> Varchar,
        version -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    multi_asset (id) {
        id -> Int8,
        policy -> Bytea,
        name -> Bytea,
        fingerprint -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    param_proposal (id) {
        id -> Int8,
        epoch_no -> Int4,
        key -> Bytea,
        min_fee_a -> Nullable<Numeric>,
        min_fee_b -> Nullable<Numeric>,
        max_block_size -> Nullable<Numeric>,
        max_tx_size -> Nullable<Numeric>,
        max_bh_size -> Nullable<Numeric>,
        key_deposit -> Nullable<Numeric>,
        pool_deposit -> Nullable<Numeric>,
        max_epoch -> Nullable<Numeric>,
        optimal_pool_count -> Nullable<Numeric>,
        influence -> Nullable<Float8>,
        monetary_expand_rate -> Nullable<Float8>,
        treasury_growth_rate -> Nullable<Float8>,
        decentralisation -> Nullable<Float8>,
        entropy -> Nullable<Bytea>,
        protocol_major -> Nullable<Int4>,
        protocol_minor -> Nullable<Int4>,
        min_utxo_value -> Nullable<Numeric>,
        min_pool_cost -> Nullable<Numeric>,
        cost_model_id -> Nullable<Int8>,
        price_mem -> Nullable<Float8>,
        price_step -> Nullable<Float8>,
        max_tx_ex_mem -> Nullable<Numeric>,
        max_tx_ex_steps -> Nullable<Numeric>,
        max_block_ex_mem -> Nullable<Numeric>,
        max_block_ex_steps -> Nullable<Numeric>,
        max_val_size -> Nullable<Numeric>,
        collateral_percent -> Nullable<Int4>,
        max_collateral_inputs -> Nullable<Int4>,
        registered_tx_id -> Int8,
        coins_per_utxo_size -> Nullable<Numeric>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pool_hash (id) {
        id -> Int8,
        hash_raw -> Bytea,
        view -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pool_metadata_ref (id) {
        id -> Int8,
        pool_id -> Int8,
        url -> Varchar,
        hash -> Bytea,
        registered_tx_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pool_offline_data (id) {
        id -> Int8,
        pool_id -> Int8,
        ticker_name -> Varchar,
        hash -> Bytea,
        json -> Jsonb,
        bytes -> Bytea,
        pmr_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pool_offline_fetch_error (id) {
        id -> Int8,
        pool_id -> Int8,
        fetch_time -> Timestamp,
        pmr_id -> Int8,
        fetch_error -> Varchar,
        retry_count -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pool_owner (id) {
        id -> Int8,
        addr_id -> Int8,
        pool_update_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pool_relay (id) {
        id -> Int8,
        update_id -> Int8,
        ipv4 -> Nullable<Varchar>,
        ipv6 -> Nullable<Varchar>,
        dns_name -> Nullable<Varchar>,
        dns_srv_name -> Nullable<Varchar>,
        port -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pool_retire (id) {
        id -> Int8,
        hash_id -> Int8,
        cert_index -> Int4,
        announced_tx_id -> Int8,
        retiring_epoch -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pool_update (id) {
        id -> Int8,
        hash_id -> Int8,
        cert_index -> Int4,
        vrf_key_hash -> Bytea,
        pledge -> Numeric,
        active_epoch_no -> Int8,
        meta_id -> Nullable<Int8>,
        margin -> Float8,
        fixed_cost -> Numeric,
        registered_tx_id -> Int8,
        reward_addr_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    pot_transfer (id) {
        id -> Int8,
        cert_index -> Int4,
        treasury -> Numeric,
        reserves -> Numeric,
        tx_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    redeemer (id) {
        id -> Int8,
        tx_id -> Int8,
        unit_mem -> Int8,
        unit_steps -> Int8,
        fee -> Nullable<Numeric>,
        purpose -> Scriptpurposetype,
        index -> Int4,
        script_hash -> Nullable<Bytea>,
        redeemer_data_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    redeemer_data (id) {
        id -> Int8,
        hash -> Bytea,
        tx_id -> Int8,
        value -> Nullable<Jsonb>,
        bytes -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    reference_tx_in (id) {
        id -> Int8,
        tx_in_id -> Int8,
        tx_out_id -> Int8,
        tx_out_index -> Int2,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    reserve (id) {
        id -> Int8,
        addr_id -> Int8,
        cert_index -> Int4,
        amount -> Numeric,
        tx_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    reserved_pool_ticker (id) {
        id -> Int8,
        name -> Varchar,
        pool_hash -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    reward (id) {
        id -> Int8,
        addr_id -> Int8,
        #[sql_name = "type"]
        type_ -> Rewardtype,
        amount -> Numeric,
        earned_epoch -> Int8,
        spendable_epoch -> Int8,
        pool_id -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    schema_version (id) {
        id -> Int8,
        stage_one -> Int8,
        stage_two -> Int8,
        stage_three -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    script (id) {
        id -> Int8,
        tx_id -> Int8,
        hash -> Bytea,
        #[sql_name = "type"]
        type_ -> Scripttype,
        json -> Nullable<Jsonb>,
        bytes -> Nullable<Bytea>,
        serialised_size -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    slot_leader (id) {
        id -> Int8,
        hash -> Bytea,
        pool_hash_id -> Nullable<Int8>,
        description -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    stake_address (id) {
        id -> Int8,
        hash_raw -> Bytea,
        view -> Varchar,
        script_hash -> Nullable<Bytea>,
        tx_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    stake_deregistration (id) {
        id -> Int8,
        addr_id -> Int8,
        cert_index -> Int4,
        epoch_no -> Int4,
        tx_id -> Int8,
        redeemer_id -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    stake_registration (id) {
        id -> Int8,
        addr_id -> Int8,
        cert_index -> Int4,
        epoch_no -> Int4,
        tx_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    treasury (id) {
        id -> Int8,
        addr_id -> Int8,
        cert_index -> Int4,
        amount -> Numeric,
        tx_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    tx (id) {
        id -> Int8,
        hash -> Bytea,
        block_id -> Int8,
        block_index -> Int4,
        out_sum -> Numeric,
        fee -> Numeric,
        deposit -> Int8,
        size -> Int4,
        invalid_before -> Nullable<Numeric>,
        invalid_hereafter -> Nullable<Numeric>,
        valid_contract -> Bool,
        script_size -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    tx_in (id) {
        id -> Int8,
        tx_in_id -> Int8,
        tx_out_id -> Int8,
        tx_out_index -> Int2,
        redeemer_id -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    tx_metadata (id) {
        id -> Int8,
        key -> Numeric,
        json -> Nullable<Jsonb>,
        bytes -> Bytea,
        tx_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    tx_out (id) {
        id -> Int8,
        tx_id -> Int8,
        index -> Int2,
        address -> Varchar,
        address_raw -> Bytea,
        address_has_script -> Bool,
        payment_cred -> Nullable<Bytea>,
        stake_address_id -> Nullable<Int8>,
        value -> Numeric,
        data_hash -> Nullable<Bytea>,
        inline_datum_id -> Nullable<Int8>,
        reference_script_id -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    withdrawal (id) {
        id -> Int8,
        addr_id -> Int8,
        amount -> Numeric,
        redeemer_id -> Nullable<Int8>,
        tx_id -> Int8,
    }
}

joinable!(ada_pots -> block (block_id));
joinable!(block -> slot_leader (slot_leader_id));
joinable!(collateral_tx_out -> datum (inline_datum_id));
joinable!(collateral_tx_out -> script (reference_script_id));
joinable!(collateral_tx_out -> stake_address (stake_address_id));
joinable!(collateral_tx_out -> tx (tx_id));
joinable!(cost_model -> block (block_id));
joinable!(datum -> tx (tx_id));
joinable!(delegation -> pool_hash (pool_hash_id));
joinable!(delegation -> redeemer (redeemer_id));
joinable!(delegation -> stake_address (addr_id));
joinable!(delegation -> tx (tx_id));
joinable!(epoch_param -> block (block_id));
joinable!(epoch_param -> cost_model (cost_model_id));
joinable!(epoch_stake -> pool_hash (pool_id));
joinable!(epoch_stake -> stake_address (addr_id));
joinable!(extra_key_witness -> tx (tx_id));
joinable!(ma_tx_mint -> multi_asset (ident));
joinable!(ma_tx_mint -> tx (tx_id));
joinable!(ma_tx_out -> multi_asset (ident));
joinable!(ma_tx_out -> tx_out (tx_out_id));
joinable!(param_proposal -> cost_model (cost_model_id));
joinable!(param_proposal -> tx (registered_tx_id));
joinable!(pool_metadata_ref -> pool_hash (pool_id));
joinable!(pool_metadata_ref -> tx (registered_tx_id));
joinable!(pool_offline_data -> pool_hash (pool_id));
joinable!(pool_offline_data -> pool_metadata_ref (pmr_id));
joinable!(pool_offline_fetch_error -> pool_hash (pool_id));
joinable!(pool_offline_fetch_error -> pool_metadata_ref (pmr_id));
joinable!(pool_owner -> pool_update (pool_update_id));
joinable!(pool_owner -> stake_address (addr_id));
joinable!(pool_relay -> pool_update (update_id));
joinable!(pool_retire -> pool_hash (hash_id));
joinable!(pool_retire -> tx (announced_tx_id));
joinable!(pool_update -> pool_hash (hash_id));
joinable!(pool_update -> pool_metadata_ref (meta_id));
joinable!(pool_update -> stake_address (reward_addr_id));
joinable!(pool_update -> tx (registered_tx_id));
joinable!(pot_transfer -> tx (tx_id));
joinable!(redeemer -> redeemer_data (redeemer_data_id));
joinable!(redeemer -> tx (tx_id));
joinable!(redeemer_data -> tx (tx_id));
joinable!(reserve -> stake_address (addr_id));
joinable!(reserve -> tx (tx_id));
joinable!(reward -> pool_hash (pool_id));
joinable!(reward -> stake_address (addr_id));
joinable!(script -> tx (tx_id));
joinable!(slot_leader -> pool_hash (pool_hash_id));
joinable!(stake_address -> tx (tx_id));
joinable!(stake_deregistration -> redeemer (redeemer_id));
joinable!(stake_deregistration -> stake_address (addr_id));
joinable!(stake_deregistration -> tx (tx_id));
joinable!(stake_registration -> stake_address (addr_id));
joinable!(stake_registration -> tx (tx_id));
joinable!(treasury -> stake_address (addr_id));
joinable!(treasury -> tx (tx_id));
joinable!(tx -> block (block_id));
joinable!(tx_in -> redeemer (redeemer_id));
joinable!(tx_metadata -> tx (tx_id));
joinable!(tx_out -> datum (inline_datum_id));
joinable!(tx_out -> script (reference_script_id));
joinable!(tx_out -> stake_address (stake_address_id));
joinable!(tx_out -> tx (tx_id));
joinable!(withdrawal -> redeemer (redeemer_id));
joinable!(withdrawal -> stake_address (addr_id));
joinable!(withdrawal -> tx (tx_id));

allow_tables_to_appear_in_same_query!(
    ada_pots,
    block,
    collateral_tx_in,
    collateral_tx_out,
    cost_model,
    datum,
    delegation,
    delisted_pool,
    epoch,
    epoch_param,
    epoch_stake,
    epoch_sync_time,
    extra_key_witness,
    ma_tx_mint,
    ma_tx_out,
    meta,
    multi_asset,
    param_proposal,
    pool_hash,
    pool_metadata_ref,
    pool_offline_data,
    pool_offline_fetch_error,
    pool_owner,
    pool_relay,
    pool_retire,
    pool_update,
    pot_transfer,
    redeemer,
    redeemer_data,
    reference_tx_in,
    reserve,
    reserved_pool_ticker,
    reward,
    schema_version,
    script,
    slot_leader,
    stake_address,
    stake_deregistration,
    stake_registration,
    treasury,
    tx,
    tx_in,
    tx_metadata,
    tx_out,
    withdrawal,
);
