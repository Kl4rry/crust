use super::read_lines::read_lines;
use std::{
    io::{stdout, Write, Stdout, BufWriter},
    fs::{File, OpenOptions},
    process::Command,
    sync::{Arc},
    path::Path,
    env,
};
use shared_child::SharedChild;
use directories::UserDirs;
use unicode_segmentation::UnicodeSegmentation;
use crossterm::{
    execute,
    queue,
    QueueableCommand,
    terminal::{
        SetTitle,
        EnableLineWrap,
        DisableLineWrap,
    },
    cursor::{
        MoveRight,
        MoveLeft,
        SavePosition,
        RestorePosition,
        MoveToColumn,
        MoveUp,
        MoveToNextLine,
        position,
    },
    style::{
        Colorize,
        Print,
        SetBackgroundColor,
        Color,
    },
    event::{
        read,
        Event,
        KeyCode,
    }
};

pub fn clear_str() -> &'static str {
    "\x1b[2J\x1b[3J\x1b[H"
}

pub struct Shell {
    stdout: Stdout,
    main_child: Arc<Option<SharedChild>>,
    history: Vec::<String>,
    history_file: BufWriter<File>,
}

impl Shell {
    pub fn new() -> Self {
        let child = Arc::new(None);

        crossterm::terminal::enable_raw_mode().unwrap();

        (execute!{
            stdout(),
            Print(clear_str()),
            SetTitle("Crust 🦀"),
            DisableLineWrap,
        }).unwrap();

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
            let name = format!("{}@{}", whoami::username().to_ascii_lowercase().yellow(), whoami::devicename().to_ascii_lowercase().red());
            write!(self.stdout, "{}", format!("{} {} {}", name, dir.to_string_lossy().green(), ">".yellow())).unwrap();
            self.stdout.flush().unwrap();

            let pos = position().unwrap();
            let start = pos.0 as usize;

            buffer.clear();
            let mut index: usize = 0;

            loop {
                match read().unwrap() {
                    Event::Key(event) => {
                        match event.code {
                            KeyCode::Char(c) => {
                                buffer.insert(index, c);
                                let string: String = buffer.iter().collect();

                                render_buffer(start, &string, index, 0, Move::Right(1));
                                self.stdout.flush().unwrap();
                                index += 1;
                            },
                            KeyCode::Backspace => {
                                if buffer.len() > 0 && index > 0 {
                                    buffer.remove(index - 1);
                                    let string: String = buffer.iter().collect();

                                    render_buffer(start, &string, index, 1, Move::Left(1));
                                    self.stdout.flush().unwrap();
                                    index -= 1;
                                }
                            },
                            KeyCode::Delete => {
                                if index < buffer.len() {
                                    buffer.remove(index);
                                    let string: String = buffer.iter().collect();
                                    
                                    render_buffer(start, &string, index, 1, Move::None);
                                    self.stdout.flush().unwrap();
                                }
                            },
                            KeyCode::Right => {
                                if index < buffer.len() {
                                    (execute!{
                                        self.stdout,
                                        MoveRight(1),
                                    }).unwrap();
                                    index += 1;
                                }
                            },
                            KeyCode::Left => {
                                if index > 0 {
                                    (execute!{
                                        self.stdout,
                                        MoveLeft(1),
                                    }).unwrap();
                                    index -= 1;
                                }
                            },
                            KeyCode::Enter => {
                                write!(self.stdout, "\n").unwrap();
                                buffer.push('\n');
                                break;
                            },
                            _ => {},
                        }
                    },
                    Event::Mouse(_) => {},
                    Event::Resize(_, _) => {},
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
                    (queue!{
                        self.stdout,
                        Print(clear_str()),
                    }).unwrap();
                }
                "pwd" => {
                    println!("{}", dir.to_string_lossy());
                }
                "exit" => {
                    return;
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
            },
            Err(_) => eprintln!("{}: command not found", cmd_name),
        };
    }
}

enum Move {
    Right(u16),
    Left(u16),
    None,
}

fn render_buffer(start: usize, buffer: &str, index: usize, removed: usize, move_dir: Move) {
    let mut temp = String::new();
    temp.extend(std::iter::repeat(' ').take(removed));

    let mut out = stdout();
    let width = crossterm::terminal::size().unwrap().0 as usize;
    let len = buffer.grapheme_indices(false).clone().count();

    /*println!("x: {}", x);
    println!("width: {}", width);
    println!("len: {}", len);
    println!("start: {}", start);*/
    //let first_row = width - start;
    let output = format!("{}{}", buffer, temp);
    let mut graphemes = output.grapheme_indices(false);
    let mut x = 0;
    
    /*if len < first_row {
        x = start + len;
        (queue!{
            out,
            SavePosition,
            MoveLeft((index) as u16),
            Print(output),
            RestorePosition,
        }).unwrap();
    } else {*/
        let rows = if (width - start) > len {
            0
        } else {
            let rest = (width - start) - len;
            rest / width
        }; 

        let mut line = String::new();
        for _ in 0..width - start {
            if let Some(glyph) = graphemes.next() {
                line.push_str(glyph.1);
            }
        }
        (queue!{
            out,
            SavePosition,
            MoveUp(rows as u16),
            MoveToColumn(start as u16 + 1),
            Print(&line),
        }).unwrap();

        for i in 1..rows {
            line.clear();

            for _ in 0..width {
                if let Some(glyph) = graphemes.next() {
                    line.push_str(glyph.1);
                } else {
                    x = i;
                    break;
                }
            }

            (queue!{
                out,
                MoveToNextLine(1),
                Print(&line),
            }).unwrap();
        }
        out.queue(RestorePosition).unwrap();
    //}
    match move_dir {
        Move::Right(distance) => {
            //println!("{}", x);
            if x >= width {
                out.queue(MoveToNextLine(1)).unwrap();
            } else {
                out.queue(MoveRight(distance)).unwrap();
            }
        },
        Move::Left(distance) => {
            out.queue(MoveLeft(distance)).unwrap();
        },
        _ => (),
    }
} 