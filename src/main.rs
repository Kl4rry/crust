mod shell;
pub use shell::parser;
use shell::Shell;

fn main() {
    ctrlc::set_handler(move || println!("sigint")).expect("Error setting Ctrl-C handler");
    let status = Shell::new().run();
    std::process::exit(status as i32);
}
