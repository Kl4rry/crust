// This file contains a modded version of the rustyline file completer
use std::{borrow::Cow, cmp, fs, path::Path, rc::Rc};

use directories::BaseDirs;
use memchr::memchr;
use rayon::prelude::*;
use rustyline::{
    completion::{escape, extract_word, unescape, Candidate, Completer, Pair, Quote},
    Context, Result,
};

use crate::shell::{
    builtins::functions::get_builtins, current_dir_path, levenshtein::levenshtein_stripped,
};

pub struct FilenameCompleter;
const DOUBLE_QUOTES_ESCAPE_CHAR: Option<char> = Some('\\');

// rl_basic_word_break_characters, rl_completer_word_break_characters
const DEFAULT_BREAK_CHARS: [u8; 18] = [
    b' ', b'\t', b'\n', b'"', b'\'', b'@', b'$', b'>', b'<', b'=', b';', b'|', b'&', b'{', b'(',
    b'\0', b'}', b')',
];

fn is_default_break_char(ch: char) -> bool {
    let Ok(ascii) = ch.try_into() else {
        return false;
    };
    memchr(ascii, &DEFAULT_BREAK_CHARS).is_some()
}

const ESCAPE_CHAR: Option<char> = Some('\\');
// In double quotes, not all break_chars need to be escaped
// https://www.gnu.org/software/bash/manual/html_node/Double-Quotes.html
const DOUBLE_QUOTES_SPECIAL_CHARS: [u8; 3] = [b'"', b'$', b'\\'];

fn is_double_quotes_special_char(ch: char) -> bool {
    let Ok(ascii) = ch.try_into() else {
        return false;
    };
    memchr(ascii, &DOUBLE_QUOTES_SPECIAL_CHARS).is_some()
}

// fn replace_escapes(line: &str, pos: usize) -> (String, usize) {
//     if line.is_empty() {
//         return (String::new(), 0);
//     }

//     let mut index = 0;
//     let mut string = String::new();
//     let mut new_pos = pos;
//     while let Some(new_index) = memchr(b'\\', &line.as_bytes()[index..]) {
//         if new_index < new_pos {
//             new_pos -= 1;
//         }
//         unsafe {
//             string
//                 .as_mut_vec()
//                 .extend_from_slice(line[index..new_index].as_bytes())
//         };

//         let escape = escape_char(*line.as_bytes().get(new_index + 1).unwrap_or(&b'\\'));
//         unsafe { string.as_mut_vec().push(escape) }
//         index = cmp::min(new_index + 2, line.len() - 1);
//     }
//     unsafe {
//         string
//             .as_mut_vec()
//             .extend_from_slice(&line.as_bytes()[index..])
//     }
//     (string, new_pos)
// }

impl FilenameCompleter {
    pub fn new() -> Self {
        Self
    }

    pub fn complete_path(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>)> {
        //let (line, pos) = replace_escapes(line, pos);
        let (start, path, esc_char, break_chars, quote): (_, _, _, fn(_: char) -> bool, _) =
            if let Some((idx, quote)) = find_unclosed_quote(&line[..pos]) {
                let start = idx + 1;
                if quote == Quote::Double {
                    (
                        start,
                        unescape(&line[start..pos], DOUBLE_QUOTES_ESCAPE_CHAR),
                        DOUBLE_QUOTES_ESCAPE_CHAR,
                        is_double_quotes_special_char,
                        quote,
                    )
                } else {
                    (
                        start,
                        Cow::Borrowed(&line[start..pos]),
                        None,
                        is_default_break_char,
                        quote,
                    )
                }
            } else {
                let (start, path) = extract_word(line, pos, None, is_default_break_char);
                (
                    start,
                    Cow::Borrowed(path),
                    ESCAPE_CHAR,
                    is_default_break_char,
                    Quote::None,
                )
            };

        let mut matches = Vec::new();
        if start == 0 && !path.contains('/') {
            matches.extend(command_complete(&path));
        }
        matches.extend(filename_complete(&path, esc_char, break_chars, quote));
        matches.par_sort_by(|a, b| {
            let start_a = a.replacement().starts_with(&*path);
            let start_b = b.replacement().starts_with(&*path);
            match start_b.cmp(&start_a) {
                cmp::Ordering::Equal => {
                    let leven_a = levenshtein_stripped(a.replacement(), &path);
                    let leven_b = levenshtein_stripped(b.replacement(), &path);
                    match leven_a.cmp(&leven_b) {
                        cmp::Ordering::Equal => a.replacement().cmp(b.replacement()),
                        ord => ord,
                    }
                }
                ord => ord,
            }
        });

        matches.dedup_by(|a, b| a.replacement() == b.replacement());

        Ok((start, matches))
    }
}

impl Completer for FilenameCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        self.complete_path(line, pos)
    }
}

fn command_complete(start: &str) -> Vec<Pair> {
    let executables = Rc::new(executable_finder::executables().unwrap());
    let mut commands: Vec<_> = executables
        .iter()
        .map(|exe| (exe.name.to_string(), Some(exe.path.clone())))
        .collect();
    // TODO add user defined functions
    commands.extend(get_builtins().map(|s| (s.to_string(), None)));
    commands.sort_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs));
    let commands = Rc::new(commands);

    let entries: Vec<Pair> = commands
        .par_iter()
        .filter_map(|(cmd, _)| {
            if cmd.starts_with(start) || levenshtein_stripped(cmd, start) < 10 {
                Some(Pair {
                    display: String::from(cmd),
                    replacement: String::from(cmd),
                })
            } else {
                None
            }
        })
        .collect();

    entries
}

fn filename_complete(
    path: &str,
    esc_char: Option<char>,
    is_break_char: fn(_: char) -> bool,
    quote: Quote,
) -> Vec<Pair> {
    #[cfg(unix)]
    let sep = '/';
    #[cfg(windows)]
    let sep = '\\';

    #[cfg(unix)]
    let path = path.to_string();

    #[cfg(windows)]
    let mut path = path.to_string();

    #[cfg(windows)]
    unsafe {
        // safe because one ascii char is replacing another ascii char
        for b in path.as_bytes_mut() {
            if *b == b'/' {
                *b = b'\\';
            }
        }
    }

    let (dir_name, file_name) = match path.rfind(sep) {
        Some(idx) => path.split_at(idx + sep.len_utf8()),
        None => ("", path.as_str()),
    };

    let dir_path = Path::new(dir_name);
    let dir = if dir_path.starts_with("~") {
        {
            if let Some(base_dirs) = BaseDirs::new() {
                let home = base_dirs.home_dir();
                match dir_path.strip_prefix("~") {
                    Ok(rel_path) => home.join(rel_path),
                    _ => home.to_path_buf(),
                }
            } else {
                dir_path.to_path_buf()
            }
        }
    } else if dir_path.is_relative() {
        current_dir_path().join(dir_path)
    } else {
        dir_path.to_path_buf()
    };

    let mut entries: Vec<Pair> = Vec::new();

    // if dir doesn't exist, then don't offer any completions
    if !dir.exists() {
        return entries;
    }

    // if any of the below IO operations have errors, just ignore them
    if let Ok(read_dir) = dir.read_dir() {
        let file_name = normalize(file_name);
        for entry in read_dir.flatten() {
            if let Some(s) = entry.file_name().to_str() {
                let ns = normalize(s);
                if ns.starts_with(file_name.as_ref()) {
                    if let Ok(metadata) = fs::metadata(entry.path()) {
                        let mut path = String::from(dir_name) + s;
                        if metadata.is_dir() {
                            path.push(sep);
                        }

                        let path = match quote {
                            Quote::Double => escape(path, esc_char, is_break_char, Quote::Double),
                            Quote::Single => path,
                            Quote::None => {
                                if path.as_bytes().iter().any(|c| is_break_char(*c as char)) {
                                    format!("'{path}'")
                                } else {
                                    path
                                }
                            }
                        };

                        entries.push(Pair {
                            display: String::from(s),
                            replacement: path,
                        });
                    }
                }
            }
        }
    }
    entries
}

#[cfg(any(windows, target_os = "macos"))]
fn normalize(s: &str) -> Cow<str> {
    // case insensitive
    Cow::Owned(s.to_lowercase())
}

#[cfg(not(any(windows, target_os = "macos")))]
fn normalize(s: &str) -> Cow<str> {
    Cow::Borrowed(s)
}

#[derive(PartialEq)]
enum ScanMode {
    DoubleQuote,
    Escape,
    EscapeInDoubleQuote,
    Normal,
    SingleQuote,
}

fn find_unclosed_quote(s: &str) -> Option<(usize, Quote)> {
    let char_indices = s.char_indices();
    let mut mode = ScanMode::Normal;
    let mut quote_index = 0;
    for (index, char) in char_indices {
        match mode {
            ScanMode::DoubleQuote => {
                if char == '"' {
                    mode = ScanMode::Normal;
                } else if char == '\\' {
                    // both windows and unix support escape in double quote
                    mode = ScanMode::EscapeInDoubleQuote;
                }
            }
            ScanMode::Escape => {
                mode = ScanMode::Normal;
            }
            ScanMode::EscapeInDoubleQuote => {
                mode = ScanMode::DoubleQuote;
            }
            ScanMode::Normal => {
                if char == '"' {
                    mode = ScanMode::DoubleQuote;
                    quote_index = index;
                } else if char == '\\' {
                    mode = ScanMode::Escape;
                } else if char == '\'' {
                    mode = ScanMode::SingleQuote;
                    quote_index = index;
                }
            }
            ScanMode::SingleQuote => {
                if char == '\'' {
                    mode = ScanMode::Normal;
                } // no escape in single quotes
            }
        };
    }
    if ScanMode::DoubleQuote == mode || ScanMode::EscapeInDoubleQuote == mode {
        return Some((quote_index, Quote::Double));
    } else if ScanMode::SingleQuote == mode {
        return Some((quote_index, Quote::Single));
    }
    None
}
