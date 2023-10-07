extern crate clap;
use clap::{App, Arg};
use std::path::Path;

mod utils; // Declare the utils module
mod notify;
mod init;
mod stop;
mod hnscraper;
mod comments_checker;

#[tokio::main]
async fn main() {

    let app = App::new("hnotify")
        .version("1.0")
        .author("Your Name")
        .about("A daemon for notifications")
        .arg(Arg::with_name("start").long("start").help("Start the notification daemon"))
        .arg(Arg::with_name("init").long("init").help("Initialize the configuration"))
        .arg(Arg::with_name("stop").long("stop").help("Stop the notification daemon"))
        .arg(Arg::with_name("help").short("h").long("help").help("Display this help message"));

    let matches = app.get_matches();

    if matches.is_present("help") {
        display_help_and_exit();
            return;
    }

    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let config_path = home_dir.join(".hnotifyrc");

    if matches.is_present("init") || (!matches.is_present("start") && !Path::new(&config_path).exists()) {
        init::init(&config_path);
    } else if matches.is_present("stop") {
        stop::stop();
    } else if matches.is_present("start") || (!matches.is_present("init") && Path::new(&config_path).exists()) {
        notify::notify().await;
    } else {
        notify::notify().await;
    }
}

fn display_help_and_exit() {
    println!(
        "USAGE:
    hnotify --start Start the notification daemon
    hnotify --init Initialize the configuration
    hnotify --stop Stop the notification daemon    
    hnotify --help Display this help message
    hnotify with no arguments is a shorthand for hnotify --start"
    );
}
