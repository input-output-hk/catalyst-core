mod load;

use load::IapyxLoadCommand;
use clap::Parser;

pub fn main() {
    IapyxLoadCommand::parse().exec().unwrap();
}
