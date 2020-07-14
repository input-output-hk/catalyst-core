use chrono::{Duration, Utc};
use rand::prelude::thread_rng;
use rand::Rng;
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
    APIToken(APITokenCmd),
}

#[derive(Debug, PartialEq, StructOpt)]
enum APITokenCmd {
    // Add token to database
    Add {
        #[structopt(long = "token")]
        token: String,

        #[structopt(long = "db-url")]
        db_url: String,
    },

    // Generate a new token
    Generate {
        #[structopt(long = "size")]
        size: usize,
    },
}

impl APITokenCmd {
    fn generate(size: usize) -> String {
        let random_bytes: Vec<u8> = (0..size).map(|_| rand::thread_rng().gen::<u8>()).collect();
        base64::encode_config(random_bytes, base64::URL_SAFE_NO_PAD)
    }

    fn add_token(base64_token: String, db_url: &str) {
        let token = base64::decode_config(base64_token, base64::URL_SAFE_NO_PAD).unwrap();
        let api_token_data = APITokenData {
            token: APIToken::new(token),
            creation_time: Utc::now().timestamp(),
            expire_time: (Utc::now() + Duration::days(365)).timestamp(),
        };
        let pool = load_db_connection_pool(db_url).unwrap();
        let connection = pool.get().unwrap();
        insert_token_data(api_token_data, &connection).unwrap();
    }
}

impl ExecTask for APITokenCmd {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<()> {
        match self {
            APITokenCmd::Add { token, db_url } => {
                APITokenCmd::add_token(token.clone(), &db_url);
            }
            APITokenCmd::Generate { size } => {
                let token = APITokenCmd::generate(*size);
                println!("{}", token);
            }
        };
        Ok(())
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
