use super::config::Monitor;
use indicatif::{ProgressBar, ProgressStyle};

pub fn use_as_monitor_progress_bar(monitor: &Monitor, title: &str, progress_bar: &mut ProgressBar) {
    let banner = format!("[Load Scenario: {}]", title);
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    progress_bar.set_style(spinner_style);
    match monitor {
        Monitor::Standard(_) => println!("{}", banner),
        Monitor::Progress(_) => {
            progress_bar.set_prefix(&banner);
            progress_bar.set_message("initializing...");
        }
        _ => (),
    };
}

pub fn use_as_status_progress_bar(progress_bar: &mut ProgressBar) {
    let spinner_style = ProgressStyle::default_spinner().template("{wide_msg}");
    progress_bar.set_style(spinner_style);
}
