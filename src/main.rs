mod shell;
use shell::*;

mod read_lines;
mod reader;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
