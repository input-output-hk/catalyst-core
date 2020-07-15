use chrono::{Duration, Utc};
use rand::Rng;
use std::io;
use structopt::StructOpt;
use vit_servicing_station_lib::{
    db::{
        load_db_connection_pool, models::api_tokens::APITokenData,
        queries::api_tokens::insert_token_data,
    },
    v0::api_token::APIToken,
};

pub trait ExecTask {
    type ResultValue;
    fn exec(&self) -> std::io::Result<<Self as ExecTask>::ResultValue>;
}

#[derive(StructOpt)]
pub enum CLIApp {
    /// API token related operations
    APIToken(APITokenCmd),
}

#[derive(Debug, PartialEq, StructOpt)]
pub enum APITokenCmd {
    /// Add provided tokens to database. If --tokens is not provided the binary will read them from the `stdin`
    Add {
        /// List of tokens in URL safe base64. If --tokens is not provided the binary will read them from the `stdin`
        #[structopt(long = "token")]
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

impl APITokenCmd {
    fn generate(n: usize, size: usize) -> Vec<String> {
        (0..n)
            .map(|_| {
                let random_bytes: Vec<u8> =
                    (0..size).map(|_| rand::thread_rng().gen::<u8>()).collect();
                base64::encode_config(random_bytes, base64::URL_SAFE_NO_PAD)
            })
            .collect()
    }

    fn add_tokens_from_stream(db_url: &str) -> io::Result<()> {
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
        APITokenCmd::add_tokens(&base64_tokens, db_url)
    }

    fn add_tokens(base64_tokens: &[String], db_url: &str) -> io::Result<()> {
        for base64_token in base64_tokens {
            let token = base64::decode_config(base64_token, base64::URL_SAFE_NO_PAD).unwrap();
            let api_token_data = APITokenData {
                token: APIToken::new(token),
                creation_time: Utc::now().timestamp(),
                expire_time: (Utc::now() + Duration::days(365)).timestamp(),
            };
            let pool = load_db_connection_pool(db_url)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))?;
            let connection = pool
                .get()
                .map_err(|e| io::Error::new(io::ErrorKind::NotConnected, format!("{}", e)))?;
            insert_token_data(api_token_data, &connection)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;
        }
        Ok(())
    }
}

impl ExecTask for APITokenCmd {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<()> {
        match self {
            APITokenCmd::Add { tokens, db_url } => match tokens {
                // if not tokens are provided then listen to stdin for input ones
                None => APITokenCmd::add_tokens_from_stream(db_url),
                // process the provided tokens
                Some(tokens) => APITokenCmd::add_tokens(tokens, &db_url),
            },
            APITokenCmd::Generate { n, size } => {
                let tokens = APITokenCmd::generate(*n, *size);
                for token in tokens {
                    println!("{}", token);
                }
                Ok(())
            }
        }
    }
}

impl ExecTask for CLIApp {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<Self::ResultValue> {
        match self {
            CLIApp::APIToken(api_token) => api_token.exec(),
        }
    }
}
