use std::{
    collections::HashMap,
    io::stdout,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

use crossterm::{execute, style::Print, terminal::SetTitle};
use directories::ProjectDirs;
use miette::{Diagnostic, GraphicalReportHandler};
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
    exit_status: i128,
    home_dir: PathBuf,
    project_dirs: ProjectDirs,
    child_id: Arc<Mutex<Option<u32>>>,
    stack: Vec<Frame>,
    aliases: HashMap<String, String>,
    recursion_limit: usize,
    interrupt: Arc<AtomicBool>,
}

impl Shell {
    pub fn new() -> Self {
        let child_id = Arc::new(Mutex::new(None));
        let handler_child = child_id.clone();
        let interrupt = Arc::new(AtomicBool::new(false));
        let handle = interrupt.clone();
        ctrlc::set_handler(move || {
            handle.store(true, Ordering::SeqCst);
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

        let project_dirs = ProjectDirs::from("", "", "crust").unwrap();

        Shell {
            running: true,
            exit_status: 0,
            home_dir,
            project_dirs,
            child_id,
            stack: vec![Frame::default()],
            aliases: HashMap::new(),
            recursion_limit: 1000,
            interrupt,
        }
    }

    pub fn run_src(mut self, src: String, name: String) -> i128 {
        let mut parser = Parser::new(src, name);
        match parser.parse() {
            Ok(ast) => {
                let res = ast.eval(&mut self);
                match res {
                    Ok(values) => {
                        print!("{}", values);
                    }
                    Err(RunTimeError::Exit) => (),
                    Err(error) => eprintln!("{}", error),
                }
            }
            Err(error) => report_error(error),
        };
        self.exit_status
    }

    pub fn run(mut self) -> i128 {
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
        let _ = editor.load_history(&self.history_path());

        while self.running {
            let readline = editor.readline(&self.promt());
            match readline {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }

                    editor.add_history_entry(&line);
                    let mut parser = Parser::new(line, String::from("shell"));
                    match parser.parse() {
                        Ok(ast) => {
                            self.interrupt.store(false, Ordering::SeqCst);
                            let res = ast.eval(&mut self);
                            match res {
                                Ok(values) => {
                                    print!("{}", values);
                                }
                                Err(RunTimeError::Exit) => (),
                                Err(error) => eprintln!("{}", error),
                            }
                        }
                        Err(error) => report_error(error),
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
        editor.save_history(&self.history_path()).unwrap();
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

    pub fn history_path(&self) -> PathBuf {
        [self.project_dirs.data_dir(), Path::new(".crust_history")]
            .iter()
            .collect()
    }

    pub fn config_path(&self) -> PathBuf {
        [self.project_dirs.data_dir(), Path::new("config.crust")]
            .iter()
            .collect()
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

pub fn report_error(error: impl Diagnostic) {
    let mut output = String::new();
    let report = GraphicalReportHandler::new();
    report.render_report(&mut output, &error).unwrap();
    eprintln!("{}", output);
}
