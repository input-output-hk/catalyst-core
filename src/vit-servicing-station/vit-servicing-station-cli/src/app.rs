use structopt::StructOpt;
pub trait ExecTask {
    type ResultValue;
    fn exec(&self) -> std::io::Result<<Self as ExecTask>::ResultValue>;
}

#[derive(StructOpt)]
pub enum CLIApp {
    APIToken(APIToken),
}

#[derive(Debug, PartialEq, StructOpt)]
enum APIToken {
    // Add token to database
    Add {
        #[structopt(long = "token")]
        token: String,
    },

    // Generate a new token
    Generate {
        #[structopt(long = "size")]
        size: usize,
    },
}

impl APIToken {
    fn generate(size: usize) -> String {
        "foo".to_string()
    }

    fn add_token(base64_token: String) {}
}

impl ExecTask for APIToken {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<()> {
        match self {
            APIToken::Add { token } => {
                APIToken::add_token(token.clone());
            }
            APIToken::Generate { size } => {
                let token = APIToken::generate(*size);
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
