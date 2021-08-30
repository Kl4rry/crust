#![feature(iter_intersperse)]
mod shell;
pub use shell::parser;
use shell::Shell;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
