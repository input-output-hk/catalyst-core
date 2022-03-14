use iapyx::cli::args::interactive::CliController;
use iapyx::cli::args::interactive::IapyxCommand;
use structopt::StructOpt;

pub fn main() {
    let controller = CliController::new().unwrap();
    IapyxCommand::from_args().exec(controller).unwrap();
}
