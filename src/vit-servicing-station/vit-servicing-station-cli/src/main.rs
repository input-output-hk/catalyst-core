mod app;

use app::*;
use structopt::StructOpt;

fn main() {
    let app = CLIApp::from_args();
    app.exec();
}
