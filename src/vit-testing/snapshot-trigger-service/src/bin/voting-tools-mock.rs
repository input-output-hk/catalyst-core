use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");

    VotingToolsCommand::from_args().exec()
}

#[derive(StructOpt, Debug)]
pub enum VotingToolsCommand {
    Genesis(self::Genesis),
}

impl VotingToolsCommand {
    pub fn exec(&self) {
        match self {
            Self::Genesis(genesis) => genesis.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct Genesis {
    #[structopt(long = "mainnet")]
    pub mainnet: bool,

    #[structopt(long = "testnet-magic")]
    pub testnet_magic: Option<u64>,

    #[structopt(long = "db")]
    pub db: String,

    #[structopt(long = "db-user")]
    pub db_user: String,

    #[structopt(long = "db-host")]
    pub db_host: PathBuf,

    #[structopt(long = "out-file")]
    pub out_file: PathBuf,

    #[structopt(long = "scale")]
    pub scale: u64,

    #[structopt(long = "slot-no")]
    pub slot_no: Option<u64>,

    #[structopt(long = "threshold")]
    pub threshold: Option<u64>,
}

impl Genesis {
    pub fn exec(&self) {
        println!("Params: {:?}", self);
        println!("slepping 5 sec..");
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!("saving {:?}", self.out_file);
        let content = "{ \
            \"initial\": [ \
                { \
                    \"fun\": [ \
                        { \
                            \"address\": \"ca1qvtkzsmvgajnagapn9ct2r5k22783qxw48jqjfjwwq683tnzcc8855t0q8u\", \
                            \"value\": 9999 \
                        }, \
                        { \
                            \"address\": \"ca1qvweqqmpxpngw6q7q5u4xt63r778nj5np6jqe5998z8el5386efy54q8eh5\", \
                            \"value\": 9672 \
                        }, \
                        { \
                            \"address\": \"ca1qvssvzgzeutayngj0gn22s9shjwwp87ndehpcqkl9vmt0chqnns25n0ksqg\", \
                            \"value\": 9999 \
                        } \
                    ] \
                } \
            ] \
        }";
        write_snapshot(content.to_string(), &self.out_file);
    }
}

pub fn write_snapshot<P: AsRef<Path>>(content: String, path: P) {
    use std::io::Write;
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}
