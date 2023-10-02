use crate::db_utils::{backup_db_file, restore_db_file};
use crate::{db_utils::db_file_exists, task::ExecTask};
use rand::Rng;
use std::collections::HashSet;
use std::io;
use structopt::StructOpt;
use thiserror::Error;
use time::{Duration, OffsetDateTime};
use vit_servicing_station_lib::{
    db::{
        load_db_connection_pool, models::api_tokens::ApiTokenData,
        queries::api_tokens::insert_token_data, DbConnection, Error as DbPoolError,
    },
    v0::api_token::ApiToken,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("base64 encoded token `{token}` is not valid")]
    Base64Decode {
        #[source]
        source: base64::DecodeError,
        token: String,
    },

    #[error("Error with database")]
    Db(#[from] diesel::result::Error),

    #[error("Error connecting db pool")]
    DbPool(#[from] DbPoolError),

    #[error("Error connecting to db")]
    DbConnection(#[from] r2d2::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, PartialEq, StructOpt)]
pub enum ApiTokenCmd {
    /// Add provided tokens to database. If --tokens is not provided the binary will read them from the `stdin`
    Add {
        /// List of tokens in URL safe base64. If --tokens is not provided the binary will read them from the `stdin`
        #[structopt(long = "tokens")]
        tokens: Option<Vec<String>>,

        /// URL of the vit-servicing-station database to interact with
        #[structopt(long = "db-url")]
        db_url: String,
    },

    /// Generate API tokens, URL safe base64 encoded.
    Generate {
        /// Number of tokens to generate
        #[structopt(long = "n", default_value = "1")]
        n: usize,

        /// Size of the token
        #[structopt(long = "size", default_value = "10")]
        size: usize,
    },
}

impl ApiTokenCmd {
    fn generate(n: usize, size: usize) -> Vec<String> {
        (0..n)
            .map(|_| {
                let random_bytes: Vec<u8> =
                    (0..size).map(|_| rand::thread_rng().gen::<u8>()).collect();
                base64::encode_config(random_bytes, base64::URL_SAFE_NO_PAD)
            })
            .collect()
    }

    fn add_tokens_from_stream(db_conn: &DbConnection) -> Result<(), Error> {
        let mut base64_tokens: Vec<String> = Vec::new();
        let mut input = String::new();
        while let Ok(n) = io::stdin().read_line(&mut input) {
            if n == 0 {
                break;
            }
            // pop the trailing `\n`
            input.pop();
            base64_tokens.push(input.clone());
        }
        ApiTokenCmd::add_tokens(&base64_tokens, db_conn)
    }

    fn add_tokens(base64_tokens: &[String], db_conn: &DbConnection) -> Result<(), Error> {
        // filter duplicated tokens
        let base64_tokens: HashSet<String> = base64_tokens.iter().cloned().collect();
        for base64_token in base64_tokens {
            let token =
                base64::decode_config(&base64_token, base64::URL_SAFE_NO_PAD).map_err(|e| {
                    Error::Base64Decode {
                        source: e,
                        token: base64_token,
                    }
                })?;
            let api_token_data = ApiTokenData {
                token: ApiToken::new(token),
                creation_time: OffsetDateTime::now_utc().unix_timestamp(),
                expire_time: (OffsetDateTime::now_utc() + Duration::days(365)).unix_timestamp(),
            };
            insert_token_data(api_token_data, db_conn).map_err(Error::Db)?;
        }
        Ok(())
    }

    fn handle_api_token_add(tokens: &Option<Vec<String>>, db_url: &str) -> Result<(), Error> {
        // check if db file exists
        db_file_exists(db_url)?;

        let pool = load_db_connection_pool(db_url).map_err(Error::DbPool)?;
        let db_conn = pool.get()?;

        match tokens {
            // if not tokens are provided then listen to stdin for input ones
            None => ApiTokenCmd::add_tokens_from_stream(&db_conn),
            // process the provided tokens
            Some(tokens) => ApiTokenCmd::add_tokens(tokens, &db_conn),
        }
    }

    fn handle_api_token_add_whith_db_backup(
        tokens: &Option<Vec<String>>,
        db_url: &str,
    ) -> Result<(), Error> {
        let backup_file = backup_db_file(db_url)?;
        if let Err(e) = Self::handle_api_token_add(tokens, db_url) {
            restore_db_file(backup_file, db_url)?;
            Err(e)
        } else {
            Ok(())
        }
    }

    fn handle_generate(n: usize, size: usize) {
        let tokens = ApiTokenCmd::generate(n, size);
        for token in tokens {
            println!("{}", token);
        }
    }
}

impl ExecTask for ApiTokenCmd {
    type ResultValue = ();
    type Error = Error;

    fn exec(&self) -> Result<(), Error> {
        match self {
            ApiTokenCmd::Add { tokens, db_url } => {
                ApiTokenCmd::handle_api_token_add_whith_db_backup(tokens, db_url)
            }
            ApiTokenCmd::Generate { n, size } => {
                ApiTokenCmd::handle_generate(*n, *size);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use vit_servicing_station_lib::db::{
        load_db_connection_pool, migrations::initialize_db_with_migration,
        queries::api_tokens::query_token_data_by_token,
    };

    #[test]
    fn generate_token() {
        let size = 10;
        let n = 10;
        let tokens = ApiTokenCmd::generate(n, size);
        assert_eq!(tokens.len(), n);
        tokens.iter().for_each(|token| {
            assert_eq!(
                base64::decode_config(token, base64::URL_SAFE_NO_PAD)
                    .unwrap()
                    .len(),
                size
            )
        })
    }

    #[test]
    fn add_token() {
        let tokens = ApiTokenCmd::generate(10, 10);
        let connection_pool = load_db_connection_pool("").unwrap();
        initialize_db_with_migration(&connection_pool.get().unwrap());
        let db_conn = connection_pool.get().unwrap();
        ApiTokenCmd::add_tokens(&tokens, &db_conn).unwrap();
        for token in tokens
            .iter()
            .map(|t| base64::decode_config(t, base64::URL_SAFE_NO_PAD).unwrap())
        {
            assert!(query_token_data_by_token(token.as_ref(), &db_conn)
                .unwrap()
                .is_some());
        }
    }
}
