mod app;

use app::*;
use structopt::StructOpt;

fn main() {
    let app = CLIApp::from_args();
    match app.exec() {
        Ok(()) => (),
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
