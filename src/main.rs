mod shell;
use shell::*;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
