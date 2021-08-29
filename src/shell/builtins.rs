use crossterm::{execute, style::Print};

use super::{clear_str, dir, Shell};

pub fn clear(shell: &mut Shell) {
    //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
    (execute! {
        shell.stdout,
        Print(clear_str()),
    })
    .unwrap();
}

pub fn pwd() {
    println!("{}", dir().to_string_lossy());
}

pub fn size() {
    let (w, h) = crossterm::terminal::size().unwrap();
    println!("{} {}", w, h);
}

pub fn exit(shell: &mut Shell) {
    shell.running = false;
}

pub fn cd(_shell: &mut Shell) {
    /*let new_dir = args.first().map_or("./", |x| *x);
    let root = Path::new(new_dir);
    if let Err(e) = env::set_current_dir(&root) {
        eprintln!("{}", e);
    }*/
}
