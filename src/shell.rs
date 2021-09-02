//#![allow(dead_code)]
use std::{
    collections::HashMap,
    io::{stdout, Stdout},
    path::PathBuf,
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
use parser::{runtime_error::RunTimeError, Parser};

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
    exit_status: i64,
    home_dir: PathBuf,
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

        let dirs = directories::UserDirs::new().unwrap();

        Shell {
            running: true,
            exit_status: 0,
            home_dir: dirs.home_dir().to_path_buf(),
            stdout: stdout(),
            main_child: child,
            variables: HashMap::new(),
        }
    }

    pub fn run(mut self) -> i64 {
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
                    if line.len() < 1 {
                        continue;
                    }

                    let mut parser = Parser::new(line.clone());
                    match parser.parse() {
                        Ok(mut ast) => {
                            let res = ast.eval(&mut self);
                            match res {
                                Ok(_value) => (),
                                Err(RunTimeError::Exit) => (),
                                Err(error) => eprintln!("{}", error),
                            }
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
        self.exit_status
    }

    fn promt(&self) -> String {
        let dir = std::env::current_dir().unwrap();
        let name = format!(
            "{}@{}",
            whoami::username().to_ascii_lowercase(),
            whoami::devicename().to_ascii_lowercase(),
        );
        let dir = dir.to_string_lossy();
        let dir =  dir.replace(self.home_dir.to_str().unwrap(), "~");
        format!("{} {} {}", name, dir, "> ",)
    }

    pub fn execute_command(&mut self, cmd_name: &str, args: &[String]) -> Result<(), std::io::Error> {
        let mut command = Command::new(cmd_name);
        command.args(args);
        let child = SharedChild::spawn(&mut command)?;
        self.main_child = Arc::new(Some(child));
        (*self.main_child).as_ref().unwrap().wait().unwrap();
        Ok(())
    }
}
