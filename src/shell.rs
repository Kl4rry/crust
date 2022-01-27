use std::{
    collections::HashMap,
    io::stdout,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use crossterm::{execute, style::Print, terminal::SetTitle};
use rustyline::{config::BellStyle, error::ReadlineError, Editor};

pub mod builtins;
pub mod parser;
pub mod stream;
pub mod value;
use parser::{runtime_error::RunTimeError, Parser};
use value::Value;
mod frame;
use frame::Frame;

mod helper;

#[inline(always)]
pub fn clear_str() -> &'static str {
    "\x1b[2J\x1b[3J\x1b[H"
}

pub struct Shell {
    running: bool,
    exit_status: i64,
    home_dir: PathBuf,
    history_file: PathBuf,
    child_id: Arc<Mutex<Option<u32>>>,
    stack: Vec<Frame>,
    aliases: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Self {
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
        .unwrap();

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
            stack: vec![Frame::new()],
            aliases: HashMap::new(),
        }
    }

    pub fn run_src(mut self, src: String) -> i64 {
        let mut parser = Parser::new(src);
        match parser.parse() {
            Ok(ast) => {
                let res = ast.eval(&mut self);
                match res {
                    Ok(_) => (),
                    Err(RunTimeError::Exit) => (),
                    Err(error) => eprintln!("{}", error),
                }
            }
            Err(error) => {
                eprintln!("{}", error)
            }
        };
        self.exit_status
    }

    pub fn run(mut self) -> i64 {
        (execute! {
            stdout(),
            Print(clear_str()),
            SetTitle("Crust 🦀"),
        })
        .unwrap();

        let config = rustyline::Config::builder()
            .color_mode(rustyline::ColorMode::Forced)
            .bell_style(BellStyle::None)
            .build();

        let mut editor = Editor::with_config(config);
        editor.set_helper(Some(helper::EditorHelper::new()));
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
                        Ok(ast) => {
                            let res = ast.eval(&mut self);
                            match res {
                                Ok(values) => {
                                    for value in values {
                                        let output = value.to_string();
                                        if !output.is_empty() {
                                            println!("{}", output);
                                        }
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

    pub fn set_child(&mut self, pid: Option<u32>) {
        *self.child_id.lock().unwrap() = pid;
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        self.stack.clear();
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}
