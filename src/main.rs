mod shell;
pub use shell::parser;
use shell::Shell;

fn main() {
    let status = Shell::new().run();
    std::process::exit(status as i32);
}
