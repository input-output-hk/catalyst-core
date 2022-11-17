CREATE TABLE tx_metadata (
    id INT8 PRIMARY KEY,
    key NUMERIC NOT NULL,
    json JSONB,
    bytes BYTEA NOT NULL,
    tx_id INT8 NOT NULL
);

CREATE TABLE tx_in (
    id INT8 PRIMARY KEY,
    tx_in_id INT8 NOT NULL,
    tx_out_id INT8 NOT NULL,
    tx_out_index INT2 NOT NULL,
    redeemer_id INT8
);

CREATE TABLE tx_out (
    id INT8 PRIMARY KEY,
    tx_id INT8 NOT NULL,
    index INT2 NOT NULL,
    address VARCHAR NOT NULL,
    address_raw BYTEA NOT NULL,
    address_has_script BOOL NOT NULL,
    payment_cred BYTEA,
    stake_address_id INT8,
    value NUMERIC NOT NULL,
    data_hash BYTEA,
    inline_datum_id INT8,
    reference_script_id INT8
);

CREATE TABLE tx (
    id INT8 PRIMARY KEY,
    hash BYTEA NOT NULL,
    block_id INT8 NOT NULL,
    block_index INT4 NOT NULL,
    out_sum NUMERIC NOT NULL,
    fee NUMERIC NOT NULL,
    deposit INT8 NOT NULL,
    size INT4 NOT NULL,
    invalid_before NUMERIC,
    invalid_hereafter NUMERIC,
    valid_contract BOOL NOT NULL,
    script_size INT4 NOT NULL
);

CREATE TABLE block (
    id INT8 PRIMARY KEY,
    hash BYTEA NOT NULL,
    epoch_no INT4,
    slot_no INT8,
    epoch_slot_no INT4,
    block_no INT4,
    previous_id INT8,
    slot_leader_id INT8 NOT NULL,
    size INT4 NOT NULL,
    time TIMESTAMP NOT NULL,
    tx_count INT8 NOT NULL,
    proto_major INT4 NOT NULL,
    proto_minor INT4 NOT NULL,
    vrf_key VARCHAR,
    op_cert BYTEA,
    op_cert_counter INT8
);
