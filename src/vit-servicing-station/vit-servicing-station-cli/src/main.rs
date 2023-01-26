mod api_token;
mod app;
mod csv;
mod db_utils;
mod init_db;
mod task;

use app::*;
use clap::Parser;
use task::ExecTask;

fn main() {
    let app = CliApp::parse();
    match app.exec() {
        Ok(()) => (),
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
