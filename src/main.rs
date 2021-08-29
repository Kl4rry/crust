mod shell;
use shell::Shell;
pub use shell::parser;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
