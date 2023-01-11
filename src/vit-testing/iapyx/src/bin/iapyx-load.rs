mod load;

use load::IapyxLoadCommand;
use clap::Parser;

pub fn main() {
    IapyxLoadCommand::from_args().exec().unwrap();
}
