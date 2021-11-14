use std::{
    collections::HashMap,
    io::stdout,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::{Arc, Mutex, MutexGuard},
};

use crossterm::{execute, style::Print, terminal::SetTitle};
use rustyline::{error::ReadlineError, Editor};

pub mod builtins;
pub mod values;
use values::HeapValue;
pub mod parser;
use parser::{runtime_error::RunTimeError, Parser};
mod frame;
use frame::Frame;
mod helper;

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
    history_file: PathBuf,
    child_id: Arc<Mutex<Option<u32>>>,
    variable_stack: Vec<Frame>,
    aliases: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Self {
        (execute! {
            stdout(),
            Print(clear_str()),
            SetTitle("Crust 🦀"),
        })
        .unwrap();

        let child_id = Arc::new(Mutex::new(None));
        let handler_child = child_id.clone();
        ctrlc::set_handler(move || {
            let guard: MutexGuard<Option<u32>> = handler_child.lock().unwrap();
            if let Some(id) = &*guard {
                #[cfg(target_family = "windows")]
                unsafe {
                    winapi::um::wincon::GenerateConsoleCtrlEvent(0, *id);
                }
                #[cfg(target_family = "unix")]
                {
                    use nix::{sys::signal, unistd::Pid};
                    signal::kill(Pid::from_raw(*id as i32), signal::Signal::SIGINT).unwrap();
                }
            }
        })
        .expect("Error setting Ctrl-C handler");

        let dirs = directories::UserDirs::new().unwrap();
        let home_dir = dirs.home_dir().to_path_buf();
        let mut history_file = home_dir.clone();
        history_file.push(".crust_history");

        Shell {
            running: true,
            exit_status: 0,
            home_dir,
            history_file,
            child_id,
            variable_stack: vec![Frame::new()],
            aliases: HashMap::new(),
        }
    }

    pub fn run(mut self) -> i64 {
        let config = rustyline::Config::builder()
            .color_mode(rustyline::ColorMode::Forced)
            .bell_style(rustyline::config::BellStyle::None)
            .build();
        let mut editor = Editor::<helper::EditorHelper>::with_config(config);
        editor.set_helper(Some(helper::EditorHelper));
        let _ = editor.load_history(&self.history_file);

        while self.running {
            let readline = editor.readline(&self.promt());
            match readline {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }

                    editor.add_history_entry(&line);
                    let mut parser = Parser::new(line);
                    match parser.parse() {
                        Ok(mut ast) => {
                            let res = ast.eval(&mut self);
                            match res {
                                Ok(values) => {
                                    for value in values {
                                        println!("{}", value.to_string());
                                    }
                                }
                                Err(RunTimeError::Exit) => (),
                                Err(error) => eprintln!("{}", error),
                            }
                        }
                        Err(error) => {
                            eprintln!("{}", error)
                        }
                    };
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    self.running = false;
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
        editor.save_history(&self.history_file).unwrap();
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
        let dir = dir.replace(self.home_dir.to_str().unwrap(), "~");
        format!("{} {} {}", name, dir, "> ",)
    }

    pub fn execute_command(
        &mut self,
        cmd_name: &str,
        args: &[String],
        piped: bool,
    ) -> Result<Output, std::io::Error> {
        let stdout = if piped {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };

        let child = Command::new(cmd_name).args(args).stdout(stdout).spawn()?;
        *self.child_id.lock().unwrap() = Some(child.id());
        let output = child.wait_with_output();
        *self.child_id.lock().unwrap() = None;
        output
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        self.variable_stack.clear();
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
