mod load;

use load::IapyxLoadCommand;
use structopt::StructOpt;

pub fn main() {
    IapyxLoadCommand::from_args().exec().unwrap();
}
