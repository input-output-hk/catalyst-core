use iapyx::cli::args::qr::IapyxQrCommand;
use structopt::StructOpt;

pub fn main() {
    IapyxQrCommand::from_args().exec().unwrap();
}
