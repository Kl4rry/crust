use crossterm::{
    execute,
    style::Print,
    terminal::SetTitle,
};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use shared_child::SharedChild;
use std::{
    env,
    io::{stdout, Stdout},
    path::Path,
    process::Command,
    sync::Arc,
};

#[inline(always)]
pub fn clear_str() -> &'static str {
    "\x1b[2J\x1b[3J\x1b[H"
}

pub struct Shell {
    stdout: Stdout,
    main_child: Arc<Option<SharedChild>>,
}

impl Shell {
    pub fn new() -> Self {
        let child = Arc::new(None);

        (execute! {
            stdout(),
            Print(clear_str()),
            SetTitle("Crust 🦀"),
        })
        .unwrap();

        Shell {
            stdout: stdout(),
            main_child: child,
        }
    }

    pub fn run(&mut self) {
        let config = rustyline::Config::builder()
            .color_mode(rustyline::ColorMode::Forced)
            .bell_style(rustyline::config::BellStyle::None)
            .build();
        let mut editor = Editor::<()>::with_config(config);
        let _ = editor.load_history("history.txt");
        
        loop {
            let dir = std::env::current_dir().unwrap();
            let name = format!(
                "{}@{}",
                whoami::username().to_ascii_lowercase(),
                whoami::devicename().to_ascii_lowercase(),
            );
            let promt = format!(
                "{} {} {}",
                name,
                dir.to_string_lossy(),
                "> ",
            );

            let readline = editor.readline(&promt);
            match readline {
                Ok(line) => {
                    editor.add_history_entry(line.as_str());
                    
                    let mut parts = line.trim().split_whitespace();
                    let command = if let Some(cmd) = parts.next() {
                        cmd
                    } else {
                        continue;
                    };
                    
                    let args: Vec<&str> = parts.collect();
                    match command {
                        "cd" => {
                            let new_dir = args.first().map_or("./", |x| *x);
                            let root = Path::new(new_dir);
                            if let Err(e) = env::set_current_dir(&root) {
                                eprintln!("{}", e);
                            }
                        }
                        "clear" => {
                            //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
                            (execute! {
                                self.stdout,
                                Print(clear_str()),
                            })
                            .unwrap();
                        }
                        "pwd" => {
                            println!("{}", dir.to_string_lossy());
                        }
                        "exit" => {
                            return;
                        }
                        "size" => {
                            let (w, h) = crossterm::terminal::size().unwrap();
                            println!("{} {}", w, h);
                        }
                        command => {
                            self.execute_command(command, &args);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        editor.save_history("history.txt").unwrap();
    }

    pub fn execute_command(&mut self, cmd_name: &str, args: &[&str]) {
        let mut command = Command::new(cmd_name);
        command.args(args);
        let shared_child = SharedChild::spawn(&mut command);

        match shared_child {
            Ok(child) => {
                self.main_child = Arc::new(Some(child));
                (*self.main_child).as_ref().unwrap().wait().unwrap();
            }
            Err(_) => eprintln!("{}: command not found", cmd_name),
        };
    }
}
