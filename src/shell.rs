use super::read_lines::read_lines;
use crossterm::{
    cursor::{
        position, MoveLeft, MoveRight, MoveToColumn, MoveToNextLine, MoveToPreviousLine, MoveUp, RestorePosition,
        SavePosition,
    },
    event::{read, Event, KeyCode},
    execute, queue,
    style::{Colorize, Print},
    terminal::{EnableLineWrap, SetTitle},
    QueueableCommand,
};
use directories::UserDirs;
use shared_child::SharedChild;
use std::{
    env,
    fs::{File, OpenOptions},
    io::{stdout, BufWriter, Stdout, Write},
    path::Path,
    process::Command,
    sync::Arc,
};
use unicode_segmentation::UnicodeSegmentation;

pub fn clear_str() -> &'static str {
    "\x1b[2J\x1b[3J\x1b[H"
}

pub struct Shell {
    stdout: Stdout,
    main_child: Arc<Option<SharedChild>>,
    history: Vec<String>,
    history_file: BufWriter<File>,
}

impl Shell {
    pub fn new() -> Self {
        let child = Arc::new(None);
        crossterm::terminal::enable_raw_mode().unwrap();

        (execute! {
            stdout(),
            Print(clear_str()),
            SetTitle("Crust 🦀"),
            EnableLineWrap,
        })
        .unwrap();

        let user_dirs = UserDirs::new().expect("unable to find user directory");
        let mut path = user_dirs
            .document_dir()
            .expect("unable to find document directory")
            .to_path_buf();
        path.push(".crust_history");

        let mut history = Vec::new();

        if let Ok(lines) = read_lines(&path) {
            for line in lines {
                if let Ok(string) = line {
                    history.push(string);
                }
            }
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();

        Shell {
            stdout: stdout(),
            main_child: child,
            history,
            history_file: BufWriter::new(file),
        }
    }

    pub fn run(&mut self) {
        let mut buffer: Vec<char> = Vec::new();

        loop {
            let dir = std::env::current_dir().unwrap();
            let name = format!(
                "{}@{}",
                whoami::username().to_ascii_lowercase().yellow(),
                whoami::devicename().to_ascii_lowercase().red()
            );
            write!(
                self.stdout,
                "{}",
                format!(
                    "{} {} {}",
                    name,
                    dir.to_string_lossy().green(),
                    ">".yellow()
                )
            )
            .unwrap();
            self.stdout.flush().unwrap();

            let pos = position().unwrap();
            let start = pos.0 as usize;

            buffer.clear();
            let mut index: usize = 0;
            let mut string = String::new();

            loop {
                string.clear();
                match read().unwrap() {
                    Event::Key(event) => match event.code {
                        KeyCode::Char(c) => {
                            buffer.insert(index, c);
                            string = buffer.iter().collect();

                            move_right(1);
                            render_buffer(start, &string, index, 0);
                            self.stdout.flush().unwrap();
                            index += 1;
                        }
                        KeyCode::Backspace => {
                            if buffer.len() > 0 && index > 0 {
                                buffer.remove(index - 1);
                                string = buffer.iter().collect();

                                move_left(1);
                                render_buffer(start, &string, index, 1);
                                self.stdout.flush().unwrap();
                                index -= 1;
                            }
                        }
                        KeyCode::Delete => {
                            if index < buffer.len() {
                                buffer.remove(index);
                                string = buffer.iter().collect();

                                render_buffer(start, &string, index, 1);
                                self.stdout.flush().unwrap();
                            }
                        }
                        KeyCode::Right => {
                            if index < buffer.len() {
                                string = buffer.iter().collect();
                                index += 1;
                                move_right(1);
                                self.stdout.flush().unwrap();
                            }
                        }
                        KeyCode::Left => {
                            if index > 0 {
                                string = buffer.iter().collect();
                                index -= 1;
                                move_left(1);
                                self.stdout.flush().unwrap();
                            }
                        }
                        KeyCode::Enter => {
                            write!(self.stdout, "\n").unwrap();
                            buffer.push('\n');
                            break;
                        }
                        _ => ()
                    },
                    Event::Mouse(_) => (),
                    Event::Resize(_, _) => (),
                }
            }

            let input: String = buffer.iter().collect();
            self.append_history(&input);
            let mut parts = input.trim().split_whitespace();
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
                    (queue! {
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
    }

    pub fn append_history(&mut self, command: &str) {
        if self.history.len() > 0 {
            if self.history.last().unwrap() == &command {
                return;
            }
        }

        self.history_file.write(command.as_bytes()).unwrap();
        self.history.push(command.to_string());
        self.history_file.flush().unwrap();
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

fn move_right(distance: usize) {
    let width = crossterm::terminal::size().unwrap().0 as usize;
    let x = position().unwrap().0 as usize;
    let mut out = stdout();

    if x >= width - 1 {
        out.queue(MoveToNextLine(1)).unwrap();
    } else {
        out.queue(MoveRight(distance as u16)).unwrap();
    }
}

fn move_left(distance: usize) {
    let width = crossterm::terminal::size().unwrap().0 as usize;
    let x = position().unwrap().0 as usize;
    let mut out = stdout();

    if x < 1 {
        (queue! {
            out,
            MoveToPreviousLine(1),
            MoveToColumn(width as u16),
        }).unwrap();
    } else {
        out.queue(MoveLeft(distance as u16)).unwrap();
    }
}

fn render_buffer(start: usize, buffer: &str, index: usize, removed: usize) {
    let width = crossterm::terminal::size().unwrap().0 as usize;
    let mut out = stdout();

    let mut temp = String::new();
    temp.extend(std::iter::repeat(' ').take(removed));
    let len = buffer.grapheme_indices(false).clone().count();

    let output = format!("{}{}", buffer, temp);
    let mut graphemes = output.grapheme_indices(false);

    let rows = if (width - start) > len {
        0
    } else {
        let short_index = index + 1 - (width - start);
        (short_index + 1) / width + 1
    };

    let mut line = String::new();
    for _ in 0..width - start {
        if let Some(glyph) = graphemes.next() {
            line.push_str(glyph.1);
        }
    }
    (queue! {
        out,
        SavePosition,
        MoveUp(rows as u16),
        MoveToColumn(start as u16 + 1),
        Print(&line),
    })
    .unwrap();

    let mut working = true;
    while working {
        line.clear();

        for _ in 0..width {
            if let Some(glyph) = graphemes.next() {
                line.push_str(glyph.1);
            } else {
                working = false;
                break;
            }
        }

        (queue! {
            out,
            MoveToNextLine(1),
            Print(&line),
        })
        .unwrap();
    }
    out.queue(RestorePosition).unwrap();
}
