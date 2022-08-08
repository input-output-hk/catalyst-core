use std::{fmt::Formatter, path::PathBuf, str::FromStr};

use clap::Parser;
use color_eyre::{eyre::bail, Report};
use serde::{de::Visitor, Deserialize, Deserializer};

use crate::config::{DbHost, DbPass, DbUser};

#[derive(Debug, Parser)]
#[cfg_attr(test, derive(PartialEq))]
#[non_exhaustive]
#[clap(about = "Create a voting power snapshot")]
pub struct Args {
    /// ID of the network to use
    #[clap(long)]
    pub network_id: NetworkId,

    /// Name of the cardano-db-sync database
    #[clap(long, default_value = "cexplorer")]
    pub db: String,

    /// User to connect to the  cardano-db-sync database with
    #[clap(long, default_value = "cexplorer")]
    pub db_user: DbUser,

    /// Host for the cardano-db-sync database connection
    #[clap(long, default_value = "/run/postgresql")]
    pub db_host: DbHost,

    /// Host for the cardano-db-sync database connection
    #[clap(long, default_value = "/run/postgresql")]
    pub db_pass: DbPass,

    /// Scale the voting funds by this amount to arrive at the voting power
    #[clap(long, default_value = "1")]
    pub scale: u64,

    /// Slot to view the state of, defaults to tip of chain. Queries registrations placed before or
    /// equal to this slot number
    #[clap(long)]
    pub slot_no: Option<u64>,

    /// File to output the signed transaction to
    #[clap(long, short = 'o')]
    pub out_file: PathBuf,
}

// TODO: is there some Rust API that can provide this type?
#[derive(Debug, PartialEq)]
pub enum NetworkId {
    MainNet,
    TestNet, // TODO: this actually should have a u32 argument
}

impl<'de> Deserialize<'de> for NetworkId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl Visitor<'_> for V {
            type Value = NetworkId;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter
                    .write_str("a case-insensitive network id (either \"mainnet\" or \"testnet\")")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                NetworkId::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(V)
    }
}

impl FromStr for NetworkId {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(NetworkId::MainNet),
            "testnet" => Ok(NetworkId::TestNet),
            s => bail!("unknown NetworkId: {s}, expected one of: [mainnet, testnet]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use microtype::SecretMicrotype;

    use super::*;

    #[test]
    fn can_parse_all_values() {
        let args = Args::parse_from([
            "binary_name",
            "--network-id",
            "mainnet",
            "--db",
            "db_name",
            "--db-user",
            "db_user",
            "--db-host",
            "localhost",
            "--db-pass",
            "super secret password",
            "--scale",
            "123",
            "--slot-no",
            "234",
            "-o",
            "some/path",
        ]);

        assert_eq!(
            args,
            Args {
                network_id: NetworkId::MainNet,
                db: "db_name".into(),
                db_user: "db_user".into(),
                db_host: "localhost".into(),
                db_pass: DbPass::new("super secret password".to_string()),
                scale: 123,
                slot_no: Some(234),
                out_file: "some/path".into()
            }
        )
    }

    #[test]
    fn can_parse_only_required_values() {
        let args = Args::parse_from(["binary_name", "-o", "some/path", "--network-id", "mainnet"]);

        assert_eq!(args.out_file, PathBuf::from("some/path"));
        assert_eq!(args.network_id, NetworkId::MainNet);
    }

    #[test]
    fn network_id_parsing() {
        assert_eq!("mAiNneT".parse::<NetworkId>().unwrap(), NetworkId::MainNet);
        assert_eq!("tEsTnEt".parse::<NetworkId>().unwrap(), NetworkId::TestNet);
        assert!("something else".parse::<NetworkId>().is_err());
    }
}
