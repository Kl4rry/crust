use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{stdout, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

use crossterm::{execute, terminal::SetTitle};
use directories::ProjectDirs;
use executable_finder::{executables, Executable};
use miette::{Diagnostic, GraphicalReportHandler};
use rustyline::{config::BellStyle, error::ReadlineError, Editor};

pub mod builtins;
pub mod parser;
pub mod stream;
pub mod value;
use parser::{shell_error::ShellErrorKind, Parser};
use subprocess::ExitStatus;
use value::Value;
mod frame;
use frame::Frame;

use self::{parser::ast::Block, stream::OutputStream};

mod helper;

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
    executables: Vec<Executable>,
    args: Vec<String>,
    prompt: Option<Block>,
}

impl Shell {
    pub fn new(args: Vec<String>) -> Self {
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

        let home_dir = directories::UserDirs::new()
            .unwrap()
            .home_dir()
            .to_path_buf();
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
            executables: executables().unwrap(),
            args,
            prompt: None,
        }
    }

    pub fn init(&mut self) -> Result<(), ShellErrorKind> {
        fs::create_dir_all(self.project_dirs.config_dir()).map_err(|e| {
            ShellErrorKind::Io(Some(self.project_dirs.config_dir().to_path_buf()), e)
        })?;
        fs::create_dir_all(self.project_dirs.data_dir())
            .map_err(|e| ShellErrorKind::Io(Some(self.project_dirs.data_dir().to_path_buf()), e))?;

        let config_path = self.config_path();
        if !config_path.is_file() {
            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&config_path)
                .map_err(|e| ShellErrorKind::Io(Some(config_path.to_path_buf()), e))?;
            f.write_all(b"# This is the crust config file")
                .map_err(|e| ShellErrorKind::Io(Some(config_path.to_path_buf()), e))?;
        }

        let config = std::fs::read_to_string(&config_path)
            .map_err(|e| ShellErrorKind::Io(Some(config_path.to_path_buf()), e))?;
        self.run_src(
            config,
            config_path.to_string_lossy().to_string(),
            &mut OutputStream::new_output(),
        );
        Ok(())
    }

    pub fn run_src(&mut self, src: String, name: String, output: &mut OutputStream) -> i128 {
        match Parser::new(src, name).parse() {
            Ok(ast) => {
                self.interrupt.store(false, Ordering::SeqCst);
                let res = ast.eval(self, output);
                if let Err(error) = res {
                    if !error.is_exit() {
                        report_error(error)
                    }
                }
            }
            Err(error) => report_error(error),
        };
        self.exit_status
    }

    pub fn run(mut self) -> Result<i128, ShellErrorKind> {
        (execute! {
            stdout(),
            SetTitle("Crust 🦀"),
        })
        .map_err(|e| ShellErrorKind::Io(None, e))?;

        let config = rustyline::Config::builder()
            .color_mode(rustyline::ColorMode::Forced)
            .bell_style(BellStyle::None)
            .build();

        let mut editor = Editor::with_config(config);
        editor.set_helper(Some(helper::EditorHelper::new()));
        let _ = editor.load_history(&self.history_path());

        let mut output = OutputStream::new_output();

        while self.running {
            let helper = editor.helper_mut().unwrap();
            helper.prompt = self.prompt().unwrap_or_else(|_| self.default_prompt());
            let stripped = console::strip_ansi_codes(&helper.prompt).to_string();

            let readline = editor.readline(&stripped);
            match readline {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }

                    editor.add_history_entry(&line);
                    self.run_src(line, String::from("shell"), &mut output);
                    output.end();
                    if let Ok((x, _)) = crossterm::cursor::position() {
                        if x != 0 {
                            println!();
                        }
                    }
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
        Ok(self.exit_status)
    }

    fn prompt(&mut self) -> Result<String, ShellErrorKind> {
        if let Some(block) = self.prompt.clone() {
            let mut output = OutputStream::new_capture();
            block.eval(self, None, None, &mut output)?;
            Ok(output.to_string())
        } else {
            Ok(self.default_prompt())
        }
    }

    fn default_prompt(&self) -> String {
        let dir = std::env::current_dir().unwrap();
        let name = format!(
            "{}@{}",
            whoami::username().to_ascii_lowercase(),
            whoami::devicename().to_ascii_lowercase(),
        );
        let dir = dir.to_string_lossy();
        let dir = dir.replace(self.home_dir.to_str().unwrap(), "~");
        format!("{} {} {}", name, dir, "> ")
    }

    pub fn history_path(&self) -> PathBuf {
        [self.project_dirs.data_dir(), Path::new(".crust_history")]
            .iter()
            .collect()
    }

    pub fn config_path(&self) -> PathBuf {
        [self.project_dirs.config_dir(), Path::new("config.crust")]
            .iter()
            .collect()
    }

    // does this functoin really need to do a linear search?
    // it could probably use a hashset instead.
    pub fn find_exe(&self, name: &str) -> Option<String> {
        for exe in &self.executables {
            if name.contains('.') || !exe.name.contains('.') {
                if name == exe.name {
                    return Some(name.to_string());
                }
            } else {
                let mut split = exe.name.split('.').rev();
                if split.next().is_some() {
                    if let Some(exe_name) = split.next() {
                        if exe_name == name {
                            return Some(exe.name.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    pub fn set_child(&mut self, pid: Option<u32>) {
        *self.child_id.lock().unwrap() = pid;
    }

    pub fn set_status(&mut self, status: ExitStatus) {
        self.exit_status = match status {
            ExitStatus::Exited(status) => status as i128,
            ExitStatus::Signaled(status) => status as i128,
            ExitStatus::Other(status) => status as i128,
            ExitStatus::Undetermined => 0,
        };
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        self.stack.clear();
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

pub fn report_error(error: impl Diagnostic) {
    let mut output = String::new();
    let report = GraphicalReportHandler::new();
    report.render_report(&mut output, &error).unwrap();
    eprintln!("{}", output);
}
