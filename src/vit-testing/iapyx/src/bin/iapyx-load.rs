mod load;

use clap::Parser;
use load::IapyxLoadCommand;

pub fn main() {
    IapyxLoadCommand::parse().exec().unwrap();
}
