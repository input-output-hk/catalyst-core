use super::fake;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

pub const PATH_TO_DYNAMIC_CONTENT: &str = "VOTING_TOOLS_DYNAMIC_CONTENT";

#[derive(StructOpt, Debug)]
pub struct VotingToolsCommand {
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
}

impl VotingToolsCommand {
    pub fn exec(&self) {
        println!("Params: {:?}", self);
        println!("sleeping 5 sec..");
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!("saving {:?}", self.out_file);

        match std::env::var(PATH_TO_DYNAMIC_CONTENT) {
            Ok(value) => {
                std::fs::copy(value, &self.out_file).unwrap();
            }
            Err(_) => {
                write_snapshot(
                    serde_json::to_string(&fake::default()).unwrap(),
                    &self.out_file,
                );
            }
        };
    }
}

pub fn write_snapshot<P: AsRef<Path>>(content: String, path: P) {
    use std::io::Write;
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}
