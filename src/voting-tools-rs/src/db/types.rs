//! Extra types defined in postgres
//!
//! db-sync defines a few types in postgres. When diesel generates a schema, it uses these types,
//! but since they don't exist on the Rust side, type checking fails. So we provide them in this
//! module
//!
//! Many types here have slightly wonky capitalization, but that's just what diesel expects

use diesel_derive_enum::DbEnum;


#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
pub enum Rewardtype {
    Leader,
    Member,
    Reserves,
    Treasury,
    Refund,
}

#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
pub enum Scriptpurposetype {
    Spend,
    Mint,
    Cert,
    Reward,
}

#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
pub enum Scripttype {
    Multisig,
    Timelock,
    PlutusV1,
    PlutusV2,
}

#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
pub enum Syncstatetype {
    Lagging,
    Following,
}


