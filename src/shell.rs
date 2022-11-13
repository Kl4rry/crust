use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

use console::Term;
use directories::{ProjectDirs, UserDirs};
use executable_finder::{executables, Executable};
use miette::{Diagnostic, GraphicalReportHandler};
use rustyline::{config::BellStyle, error::ReadlineError, Editor};
use yansi::Paint;

pub mod builtins;
pub mod parser;
pub mod stream;
pub mod value;
use parser::{shell_error::ShellErrorKind, Parser};
use subprocess::ExitStatus;
use value::Value;
mod frame;
use frame::Frame;
mod hello;

use self::{
    helper::EditorHelper,
    parser::{ast::context::Context, shell_error::ShellError},
    stream::OutputStream,
};

mod helper;
mod levenshtein;

pub struct Shell {
    running: bool,
    exit_status: i64,
    user_dirs: UserDirs,
    project_dirs: ProjectDirs,
    child_id: Arc<Mutex<Option<u32>>>,
    stack: Frame,
    aliases: HashMap<String, String>,
    recursion_limit: usize,
    interrupt: Arc<AtomicBool>,
    executables: Rc<Vec<Executable>>,
    args: Vec<String>,
    editor: Editor<EditorHelper>,
    interactive: bool,
}

impl Shell {
    pub fn new(args: Vec<String>) -> Self {
        let child_id = Arc::new(Mutex::new(None));
        let handler_child = child_id.clone();
        let interrupt = Arc::new(AtomicBool::new(false));
        let handle = interrupt.clone();
        ctrlc::set_handler(move || {
            handle.store(true, Ordering::SeqCst);
            let mut guard: MutexGuard<Option<u32>> = handler_child.lock().unwrap();
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
                *guard = None;
            }
        })
        .unwrap();

        let executables = Rc::new(executables().unwrap());

        let project_dirs = ProjectDirs::from("", "", "crust").unwrap();
        let user_dirs = UserDirs::new().unwrap();

        let config = rustyline::Config::builder()
            .max_history_size(5000)
            .color_mode(rustyline::ColorMode::Enabled)
            .bell_style(BellStyle::None)
            .build();

        let mut editor = Editor::with_config(config).unwrap();
        editor.set_helper(Some(helper::EditorHelper::new()));
        let _ = editor.load_history(&history_path(&project_dirs));

        Shell {
            running: true,
            exit_status: 0,
            user_dirs,
            project_dirs,
            child_id,
            stack: Frame::default(),
            aliases: HashMap::new(),
            recursion_limit: 1000,
            interrupt,
            executables,
            args,
            editor,
            interactive: false,
        }
    }

    fn load_env(&mut self) {
        for (key, value) in std::env::vars() {
            self.stack.add_env_var(key, Value::from(value));
        }
    }

    pub fn init(&mut self) -> Result<(), ShellErrorKind> {
        self.load_env();
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
            f.write_all(include_bytes!("../config/default.crust"))
                .map_err(|e| ShellErrorKind::Io(Some(config_path.to_path_buf()), e))?;
        }

        let config = std::fs::read_to_string(&config_path)
            .map_err(|e| ShellErrorKind::Io(Some(config_path.to_path_buf()), e))?;
        let mut output = OutputStream::new_output();
        self.run_src(
            config_path.to_string_lossy().to_string(),
            config,
            &mut output,
        );
        output.end();
        Ok(())
    }

    pub fn run_src(&mut self, name: String, src: String, output: &mut OutputStream) {
        match Parser::new(name, src).parse() {
            Ok(ast) => {
                let res = ast.eval(self, output);
                if let Err(error) = res {
                    if !error.is_exit() {
                        report_error(error)
                    }
                }
            }
            Err(error) => report_error(*error),
        };
    }

    pub fn run(mut self) -> Result<i64, ShellErrorKind> {
        hello::hello();
        let term = Term::stdout();
        while self.running {
            self.interrupt.store(false, Ordering::SeqCst);

            #[cfg(debug_assertions)]
            let info = " (DEBUG)";
            #[cfg(not(debug_assertions))]
            let info =
                current_dir_str().replace(&self.home_dir().to_string_lossy().to_string(), "~");

            term.set_title(format!("Crust: {info}"));

            self.editor.helper_mut().unwrap().prompt = self.prompt();
            let stripped =
                console::strip_ansi_codes(&self.editor.helper_mut().unwrap().prompt).to_string();

            let mut output = OutputStream::new_output();

            let readline = self.editor.readline(&stripped);
            match readline {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }

                    self.editor.add_history_entry(&line);
                    self.save_history();
                    self.run_src(String::from("shell"), line, &mut output);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", Paint::red("^C"));
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", Paint::red("^D"));
                    self.running = false;
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
            output.end();
            reset_cursor()
        }
        Ok(self.exit_status)
    }

    fn prompt(&mut self) -> String {
        if let Some(func) = self.stack.get_function("prompt") {
            if func.parameters.is_empty() {
                let mut output = OutputStream::new_capture();
                let mut ctx = Context {
                    frame: self.stack.clone(),
                    shell: self,
                    output: &mut output,
                    src: func.src.clone(),
                };

                match func.block.eval(&mut ctx, None, None) {
                    Ok(_) => return output.to_string(),
                    Err(err) => report_error(ShellError::new(
                        err,
                        func.src.clone(),
                        self.executables.clone(),
                    )),
                }
            }
        }
        self.default_prompt()
    }

    fn default_prompt(&self) -> String {
        let name = format!(
            "{}@{}",
            whoami::username().to_ascii_lowercase(),
            whoami::devicename().to_ascii_lowercase(),
        );
        let dir = current_dir_str().replace(&*self.home_dir().to_string_lossy(), "~");
        format!("{} {} {}", name, dir, "> ")
    }

    pub fn history_path(&self) -> PathBuf {
        history_path(&self.project_dirs)
    }

    pub fn config_path(&self) -> PathBuf {
        [self.project_dirs.config_dir(), Path::new("config.crust")]
            .iter()
            .collect::<PathBuf>()
    }

    pub fn home_dir(&self) -> PathBuf {
        self.user_dirs.home_dir().to_path_buf()
    }

    // does this function really need to do a linear search?
    // it could probably use a hashset instead.
    pub fn find_exe(&self, name: &str) -> Option<String> {
        for exe in self.executables.iter() {
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
            ExitStatus::Exited(status) => status as i64,
            ExitStatus::Signaled(status) => status as i64,
            ExitStatus::Other(status) => status as i64,
            ExitStatus::Undetermined => 0,
        };
    }

    pub fn status(&self) -> i64 {
        self.exit_status
    }

    fn save_history(&mut self) {
        let _ = self.editor.append_history(&self.history_path());
    }

    pub fn set_interactive(&mut self, interactive: bool) {
        self.interactive = interactive;
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

pub fn current_dir_path() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub fn current_dir_str() -> String {
    current_dir_path().to_string_lossy().to_string()
}

pub fn history_path(project_dirs: &ProjectDirs) -> PathBuf {
    [project_dirs.data_dir(), Path::new(".crust_history")]
        .iter()
        .collect::<PathBuf>()
}

pub fn report_error(error: impl Diagnostic) {
    reset_cursor();
    let mut output = String::new();
    let report = GraphicalReportHandler::new();
    report.render_report(&mut output, &error).unwrap();
    eprintln!("{}", output);
}

pub fn reset_cursor() {
    if let Ok((x, _)) = crossterm::cursor::position() {
        if x != 0 {
            println!();
        }
    }
}
