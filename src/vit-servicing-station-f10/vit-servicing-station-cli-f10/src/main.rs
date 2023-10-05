mod api_token;
mod app;
mod csv;
mod db_utils;
mod init_db;
mod task;

use app::*;
use structopt::StructOpt;
use task::ExecTask;

fn main() {
    let app = CliApp::from_args();
    match app.exec() {
        Ok(()) => (),
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
