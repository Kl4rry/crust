#![allow(dead_code)]
#![allow(unused_imports)]
use std::{
    collections::HashMap,
    env,
    io::{stdout, Stdout},
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::Arc,
};

use crossterm::{execute, style::Print, terminal::SetTitle};
use rustyline::{error::ReadlineError, Editor};
use shared_child::SharedChild;

pub mod builtins;
pub mod gc;
pub mod parser;
use parser::Parser;

#[inline(always)]
pub fn clear_str() -> &'static str {
    "\x1b[2J\x1b[3J\x1b[H"
}

#[inline(always)]
fn dir() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub struct Shell {
    running: bool,
    stdout: Stdout,
    main_child: Arc<Option<SharedChild>>,
    variables: HashMap<String, Rc<gc::Value>>,
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
            running: true,
            stdout: stdout(),
            main_child: child,
            variables: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let config = rustyline::Config::builder()
            .color_mode(rustyline::ColorMode::Forced)
            .bell_style(rustyline::config::BellStyle::None)
            .build();
        let mut editor = Editor::<()>::with_config(config);
        let _ = editor.load_history("history.txt");

        while self.running {
            let readline = editor.readline(&self.promt());
            match readline {
                Ok(line) => {
                    let mut parser = Parser::new(line.clone());
                    match parser.parse() {
                        Ok(ast) => {
                            println!("{:?}", ast);
                        }
                        Err(error) => {
                            eprintln!("{}", error)
                        }
                    };
                    editor.add_history_entry(line.as_str());
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    self.running = false;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
        editor.save_history("history.txt").unwrap();
    }

    fn promt(&self) -> String {
        let dir = std::env::current_dir().unwrap();
        let name = format!(
            "{}@{}",
            whoami::username().to_ascii_lowercase(),
            whoami::devicename().to_ascii_lowercase(),
        );
        format!("{} {} {}", name, dir.to_string_lossy(), "> ",)
    }

    pub fn _execute_command(&mut self, cmd_name: &str, args: &[&str]) {
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
