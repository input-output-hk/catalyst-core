use std::fs::File;

use bigdecimal::BigDecimal;
use diesel::{delete, insert_into, RunQueryDsl};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    db::schema::{tx_in, tx_metadata, tx_out},
    model::{DbHost, DbPass, DbUser},
    Db, DbConfig,
};

/// Get a handle to the reference db for testing
///
/// This function reads a file at `<project-root>/test_db.json`. The contents of this file
/// should be a json, for example:
/// ```json
/// {
///   "host": "localhost",
///   "user": "username",
///   "password", "password",
/// }
/// ```
/// If the database doesn't exist, it will be created and populated. If it does, a handle will be
/// returned. This function is safe to be called from multiple threads
pub fn reference_db() -> &'static Db {
    &*REFERENCE_DB
}

static REFERENCE_DB: Lazy<Db> = Lazy::new(critical_section);

const DB_NAME: &str = "voting_tools_rs_reference";

#[derive(Deserialize)]
struct RefDb {
    host: DbHost,
    user: DbUser,
    password: Option<DbPass>,
}

/// This is guaranteed by [`Lazy`] to only be called once. It will wipe the relevant tables
fn critical_section() -> Db {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_db.json");
    let file = File::open(path).unwrap();
    let RefDb {
        host,
        user,
        password,
    } = serde_json::from_reader(file).unwrap();

    let config = DbConfig {
        host,
        user,
        password,
        name: DB_NAME.into(),
    };

    create_db_fresh(&config);
    let db = Db::connect(config).unwrap();

    initialize(&db);

    db
}

fn create_db_fresh(
    DbConfig {
        host, user, name, ..
    }: &DbConfig,
) {
    use postgres::{Client, NoTls};

    let mut client = Client::connect(&format!("host={host} user={user}"), NoTls).unwrap();
    client
        .execute(&format!("DROP DATABASE IF EXISTS {DB_NAME};"), &[])
        .unwrap();
    client
        .execute(&format!("CREATE DATABASE {DB_NAME};"), &[])
        .unwrap();

    let mut client = Client::configure()
        .host(host)
        .user(user)
        .dbname(name)
        .connect(NoTls)
        .unwrap();

    client
        .batch_execute(
            r#"
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

        "#,
        )
        .unwrap();
}

fn initialize(db: &Db) {
    db.exec(|conn| {
        delete(tx_metadata::table).execute(conn)?;
        insert_into(tx_metadata::table)
            .values(reference_tx_metadata())
            .execute(conn)?;
        Ok(())
    })
    .unwrap();

    db.exec(|conn| {
        delete(tx_in::table).execute(conn)?;
        insert_into(tx_in::table)
            .values(reference_tx_in())
            .execute(conn)?;
        Ok(())
    })
    .unwrap();

    db.exec(|conn| {
        delete(tx_out::table).execute(conn)?;
        insert_into(tx_out::table)
            .values(reference_tx_out())
            .execute(conn)?;
        Ok(())
    })
    .unwrap();
}

#[derive(Debug, PartialEq, Deserialize, Queryable, Insertable)]
#[diesel(table_name = tx_metadata)]
struct TxMetaRow {
    id: i64,
    key: BigDecimal,
    json: Option<Value>,
    bytes: Vec<u8>,
    tx_id: i64,
}

fn reference_tx_metadata() -> Vec<TxMetaRow> {
    serde_json::from_str(include_str!("reference_tx_meta.json")).unwrap()
}

#[derive(Debug, PartialEq, Deserialize, Queryable, Insertable)]
#[diesel(table_name = tx_in)]
struct TxInRow {
    id: i64,
    tx_in_id: i64,
    tx_out_id: i64,
    tx_out_index: i16,
    redeemer_id: Option<i64>,
}

fn reference_tx_in() -> Vec<TxInRow> {
    serde_json::from_str(include_str!("reference_tx_in.json")).unwrap()
}

#[derive(Debug, PartialEq, Deserialize, Queryable, Insertable)]
#[diesel(table_name = tx_out)]
struct TxOutRow {
    id: i64,
    tx_id: i64,
    index: i16,
    address: String,
    address_raw: Vec<u8>,
    address_has_script: bool,
    payment_cred: Option<Vec<u8>>,
    stake_address_id: Option<i64>,
    value: BigDecimal,
    data_hash: Option<Vec<u8>>,
    inline_datum_id: Option<i64>,
    reference_script_id: Option<i64>,
}

fn reference_tx_out() -> Vec<TxOutRow> {
    serde_json::from_str(include_str!("reference_tx_out.json")).unwrap()
}
