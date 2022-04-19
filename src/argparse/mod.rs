#![allow(unused)]
#![allow(clippy::needless_borrow)]
use std::{
    cmp,
    collections::{HashMap, VecDeque},
    error::Error,
    fmt,
    fmt::Write,
    iter::Peekable,
    rc::Rc,
};

use unicode_width::UnicodeWidthStr;
use yansi::Paint;

mod arg;
pub use arg::Arg;

mod opt;
pub use opt::Opt;

mod flag;
pub use flag::Flag;

use crate::shell::value::{Type, Value};
// sub commands does not exist yet
#[derive(Debug)]
pub struct App {
    name: String,
    about: String,
    args: Vec<Arg>,
    options: Vec<Opt>,
    flags: Vec<Flag>,
}

impl App {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();

        App {
            name,
            about: String::new(),
            args: Vec::new(),
            options: Vec::new(),
            flags: Vec::new(),
        }
    }

    pub fn about(mut self, about: &str) -> Self {
        self.about.clear();
        self.about.push_str(about);
        self
    }

    pub fn arg(mut self, arg: Arg) -> Self {
        if arg.multiple && self.args.iter().any(|a| a.multiple) {
            panic!("only one positional argument can take multiple input");
        }

        if self.args.iter().any(|a| a.multiple) {
            panic!("multiple must be last");
        }

        if arg.required && self.args.iter().any(|a| !a.required) {
            panic!("required args must always be first");
        }

        if self.args.iter().any(|a| a.name == arg.name)
            && self.options.iter().any(|a| a.name == arg.name)
            && self.flags.iter().any(|a| a.name == arg.name)
        {
            panic!("arg names must be unique");
        }

        self.args.push(arg);
        self
    }

    pub fn opt(mut self, opt: Opt) -> Self {
        if self.validate_naming(&self.name, opt.long.as_deref(), opt.short) {
            panic!("invalid option");
        }

        self.options.push(opt);
        self
    }

    pub fn flag(mut self, flag: Flag) -> Self {
        if self.validate_naming(&self.name, flag.long.as_deref(), flag.short) {
            panic!("invalid flag");
        }

        self.flags.push(flag);
        self
    }

    fn validate_naming(&self, name: &str, long: Option<&str>, short: Option<char>) -> bool {
        if long.is_none() && short.is_none() {
            return true;
        }

        if self.flags.iter().any(|f| {
            f.name == name
                || (f.short == short && (f.short.is_some() || short.is_some()))
                || (f.long.as_deref() == long && (f.long.is_some() || long.is_some()))
        }) {
            return true;
        }

        if self.options.iter().any(|f| {
            f.name == name
                || (f.short == short && (f.short.is_some() || short.is_some()))
                || (f.long.as_deref() == long && (f.long.is_some() || long.is_some()))
        }) {
            return true;
        }

        if self.args.iter().any(|f| f.name == name) {
            return true;
        }

        if short == Some('h') || short == Some('-') || long == Some("help") {
            return true;
        }

        false
    }

    pub fn usage(&self) -> String {
        let mut output = String::new();
        write!(
            output,
            "{}\n    {}",
            Paint::yellow("Usage:"),
            Paint::green(&self.name)
        )
        .unwrap();
        write!(output, " [FLAGS]").unwrap();

        if !self.options.is_empty() && self.options.iter().any(|o| !o.required) {
            write!(output, " [OPTIONS]").unwrap();
        }

        for opt in self.options.iter().filter(|o| o.required) {
            write!(output, " {}", opt).unwrap()
        }

        // if this is not printed all args even optional should be printed
        if self.options.iter().any(|o| o.multiple) && !self.args.is_empty() {
            write!(output, " [--]").unwrap();
        }

        let mut argc = 0;
        for arg in self.args.iter().filter(|o| o.required) {
            write!(output, " {}", arg).unwrap();
            argc += 1;
        }

        if argc < self.args.len() {
            if self.args.len() - argc == 1 {
                let arg = self.args.iter().find(|a| !a.required).unwrap();
                write!(output, " {arg}").unwrap();
            } else {
                write!(output, " [ARGS]").unwrap();
            }
        }
        output
    }

    fn help(&self) -> String {
        let mut output = String::new();

        {
            writeln!(output, "{}\n", self.about).unwrap();
            write!(output, "{}", self.usage()).unwrap();
            writeln!(output).unwrap();

            writeln!(output, "\n{}", Paint::yellow("Flags:")).unwrap();
            let mut strs = Vec::new();
            let mut helps = Vec::new();
            let mut width: usize = 0;

            for flag in self.flags.iter() {
                let mut temp = String::new();

                let short = match flag.short {
                    Some(short) => {
                        write!(temp, "-{}", short).unwrap();
                        true
                    }
                    None => false,
                };

                if let Some(long) = &flag.long {
                    if short {
                        write!(temp, ", ").unwrap();
                    }
                    write!(temp, "--{}", long).unwrap();
                }
                width = cmp::max(width, temp.width());
                strs.push(temp);
                helps.push(flag.help.as_str());
            }

            strs.push(String::from("-h, --help"));
            helps.push("Display this help message");
            // safe because we just pushed something to strs
            width = cmp::max(width, unsafe { strs.last().unwrap_unchecked() }.width());

            for (help, flag_str) in helps.iter().zip(strs.iter()) {
                writeln!(
                    output,
                    "    {:width$}    {}",
                    Paint::green(flag_str),
                    help,
                    width = width
                )
                .unwrap();
            }
        }

        if !self.options.is_empty() {
            writeln!(output, "\n{}", Paint::yellow("Options:")).unwrap();
            let mut strs = Vec::new();
            let mut width: usize = 0;
            for option in self.options.iter() {
                let mut temp = String::new();
                let short = match option.short {
                    Some(short) => {
                        write!(temp, "-{}", short).unwrap();
                        true
                    }
                    None => false,
                };

                if let Some(long) = &option.long {
                    if short {
                        write!(temp, ", ").unwrap();
                    }
                    write!(temp, "--{}", long).unwrap();
                }
                write!(temp, " <{}>", option.name).unwrap();
                width = cmp::max(width, temp.width());
                strs.push(temp);
            }
            for (option, option_str) in self.options.iter().zip(strs.iter()) {
                writeln!(
                    output,
                    "    {:width$}    {}",
                    Paint::green(option_str),
                    option.help,
                    width = width
                )
                .unwrap();
            }
        }

        if !self.args.is_empty() {
            writeln!(output, "\n{}", Paint::yellow("Args:")).unwrap();
            let mut strs = Vec::new();
            let mut width: usize = 0;
            for arg in self.args.iter() {
                let mut temp = String::new();
                write!(temp, " <{}>", arg.name).unwrap();
                if arg.multiple {
                    write!(temp, " ...").unwrap();
                }
                width = cmp::max(width, temp.width());
                strs.push(temp);
            }
            for (p, s) in self.args.iter().zip(strs.iter()) {
                writeln!(
                    output,
                    "   {:width$}    {}",
                    Paint::green(s),
                    p.help,
                    width = width
                )
                .unwrap();
            }
        }

        output
    }

    pub fn parse(&self, args: impl Iterator<Item = Value>) -> Result<Matches, ParseError> {
        let parser = Parser::new(self, args);
        parser.parse().map_err(|e| ParseError::new(self, e))
    }
}

#[derive(Debug)]
pub enum ParseErrorKind {
    Help(Value),
    MissingArgs(Vec<String>),
    InvalidInContext(String),
    TakesValue(String),
    WrongType {
        name: String,
        expected: Type,
        recived: Type,
    },
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Help(s) => write!(f, "{}", s),
            Self::MissingArgs(s) => write!(
                f,
                "The following required arguments were not provided:\n    {}",
                s.join("\n    ")
            ),
            Self::InvalidInContext(s) => write!(
                f,
                "Found argument '{s}' which wasn't expected, or isn't valid in this context"
            ),
            Self::TakesValue(s) => write!(
                f,
                "The argument '{s}' requires a value but none was supplied"
            ),
            Self::WrongType {
                name,
                expected,
                recived,
            } => write!(
                f,
                "{name} expected value of type {expected} but recived {recived}",
            ),
        }
    }
}

impl Error for ParseErrorKind {}

#[derive(Debug)]
pub struct ParseError {
    usage: String,
    pub error: ParseErrorKind,
}

impl ParseError {
    fn new(app: &App, error: ParseErrorKind) -> Self {
        let usage = app.usage();
        Self { usage, error }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}\n", self.error)?;
        if !matches!(self.error, ParseErrorKind::Help(_)) {
            writeln!(f, "{}\n", self.usage)?;
        }
        Ok(())
    }
}

impl Error for ParseError {}

#[derive(Debug)]
struct Parser<'a, T: Iterator<Item = Value>> {
    app: &'a App,
    args: Peekable<T>,
    arg_index: usize,
    matches: Matches,
}

impl<'a, T> Parser<'a, T>
where
    T: Iterator<Item = Value>,
{
    fn new(app: &'a App, args: T) -> Self {
        Self {
            app,
            args: args.peekable(),
            arg_index: 0,
            matches: Matches::default(),
        }
    }

    fn parse(mut self) -> Result<Matches, ParseErrorKind> {
        while let Some(arg) = self.args.peek() {
            if let Value::String(arg) = arg {
                if arg.is_empty() {
                    if let Some(arg) = self.app.args.get(self.arg_index) {
                        if arg.multiple {
                            self.parse_args()?;
                        } else {
                            self.parse_arg()?;
                        }
                        continue;
                    }
                }
            }

            if let Value::String(arg) = arg {
                let mut arg_iter = arg.bytes();
                if arg_iter.next().unwrap() == b'-' {
                    match arg_iter.next() {
                        Some(b'-') => loop {
                            match arg_iter.next() {
                                Some(b'-') => (),
                                Some(symbol) => {
                                    let mut bytes = vec![symbol];
                                    bytes.extend(arg_iter);
                                    let long = unsafe { String::from_utf8_unchecked(bytes) };
                                    if long == "help" {
                                        return Err(ParseErrorKind::Help(Value::String(Rc::new(
                                            self.app.help(),
                                        ))));
                                    } else if let Some(opt) = self
                                        .app
                                        .options
                                        .iter()
                                        .find(|i| i.long.as_ref() == Some(&long))
                                    {
                                        self.args.next();
                                        self.parse_option(opt)?;
                                    } else if let Some(flag) = self
                                        .app
                                        .flags
                                        .iter()
                                        .find(|i| i.long.as_ref() == Some(&long))
                                    {
                                        self.args.next();
                                        self.parse_flag(flag);
                                    } else {
                                        return Err(ParseErrorKind::InvalidInContext(long));
                                    }
                                    break;
                                }
                                None => {
                                    self.parse_arg()?;
                                    break;
                                }
                            }
                        },
                        Some(_) => {
                            self.parse_short()?;
                            continue;
                        }
                        None => {
                            self.parse_arg()?;
                            continue;
                        }
                    }
                }
            }
            self.parse_arg()?
        }

        let mut missing_args = Vec::new();
        for arg in &self.app.args {
            if arg.required && !self.matches.args.contains_key(&arg.name) {
                missing_args.push(arg.to_string());
            }
        }

        for opt in &self.app.options {
            if opt.required && !self.matches.args.contains_key(&opt.name) {
                missing_args.push(opt.to_string());
            }
        }

        if !missing_args.is_empty() {
            return Err(ParseErrorKind::MissingArgs(missing_args));
        }

        // This typecheck impl is quite lazy
        for arg in &self.app.args {
            if let Some(m) = self.matches.get(&arg.name) {
                for v in &m.values {
                    if !v.to_type().intersects(arg.value) {
                        return Err(ParseErrorKind::WrongType {
                            name: arg.to_string(),
                            expected: arg.value,
                            recived: v.to_type(),
                        });
                    }
                }
            }
        }

        for opt in &self.app.options {
            if let Some(m) = self.matches.get(&opt.name) {
                if m.values.iter().any(|v| v.to_type() != opt.value) {}
                for v in &m.values {
                    if v.to_type() != opt.value {
                        return Err(ParseErrorKind::WrongType {
                            name: opt.to_string(),
                            expected: opt.value,
                            recived: v.to_type(),
                        });
                    }
                }
            }
        }

        let Parser { matches, .. } = self;
        Ok(matches)
    }

    fn parse_short(&mut self) -> Result<(), ParseErrorKind> {
        let slice = &self.args.next().unwrap().unwrap_string()[1..];
        let mut chars = slice.chars();
        while let Some(c) = chars.next() {
            if c == '-' {
                return Err(ParseErrorKind::InvalidInContext(String::from("-")));
            }

            if c == 'h' {
                return Err(ParseErrorKind::Help(Value::String(Rc::new(
                    self.app.help(),
                ))));
            } else if let Some(option) = self.app.options.iter().find(|o| o.short == Some(c)) {
                let rest: String = chars.collect();

                if rest.is_empty() {
                    self.parse_option(option)?;
                } else {
                    // do some weridness here to get parse option to work
                    // this is very nasty code duping
                    let arg_match = self
                        .matches
                        .args
                        .entry(option.name.clone())
                        .or_insert_with(ArgMatch::default);

                    if option.multiple {
                        arg_match.values.push_back(self.args.next().unwrap());
                        self.parse_option(&option)?;
                    } else {
                        arg_match.values.push_back(self.args.next().unwrap());
                        arg_match.occurs += 1;
                    }
                }

                break;
            } else if let Some(flag) = self.app.flags.iter().find(|o| o.short == Some(c)) {
                self.parse_flag(flag);
            } else {
                return Err(ParseErrorKind::InvalidInContext(String::from(c)));
            }
        }
        Ok(())
    }

    fn parse_option(&mut self, option: &Opt) -> Result<(), ParseErrorKind> {
        let arg_match = self
            .matches
            .args
            .entry(option.name.clone())
            .or_insert_with(ArgMatch::default);

        if option.multiple {
            while let Some(arg) = self.args.peek() {
                if let Value::String(arg) = arg {
                    if arg.starts_with('-') {
                        if arg.bytes().all(|i| i == b'-') {
                            if arg.len() == 2 {
                                self.args.next();
                            } else {
                                return Err(ParseErrorKind::InvalidInContext(arg.to_string()));
                            }
                        }
                        break;
                    }
                    arg_match.values.push_back(self.args.next().unwrap());
                }
            }
        } else if let Some(arg) = self.args.peek() {
            match arg {
                Value::String(s) if !s.starts_with('-') => {
                    arg_match.values.push_back(self.args.next().unwrap())
                }
                Value::String(_) => (),
                value => arg_match.values.push_back(self.args.next().unwrap()),
            }
        }

        if arg_match.values.is_empty() {
            return Err(ParseErrorKind::TakesValue(option.to_string()));
        }
        arg_match.occurs += 1;
        Ok(())
    }

    fn parse_flag(&mut self, flag: &Flag) {
        let arg_match = self
            .matches
            .args
            .entry(flag.name.clone())
            .or_insert_with(ArgMatch::default);
        arg_match.occurs += 1;
    }

    fn parse_arg(&mut self) -> Result<(), ParseErrorKind> {
        let arg = match self.app.args.get(self.arg_index) {
            Some(arg) => arg,
            None => {
                return Err(ParseErrorKind::InvalidInContext(
                    self.args.next().unwrap().to_string(),
                ))
            }
        };
        let arg_match = self
            .matches
            .args
            .entry(arg.name.clone())
            .or_insert_with(ArgMatch::default);
        arg_match.values.push_back(self.args.next().unwrap());
        arg_match.occurs += 1;
        self.arg_index += 1;
        Ok(())
    }

    fn parse_args(&mut self) -> Result<(), ParseErrorKind> {
        let arg = match self.app.args.get(self.arg_index) {
            Some(arg) => arg,
            None => {
                return Err(ParseErrorKind::InvalidInContext(
                    self.args.next().unwrap().to_string(),
                ))
            }
        };
        let arg_match = self
            .matches
            .args
            .entry(arg.name.clone())
            .or_insert_with(ArgMatch::default);
        for arg in self.args.by_ref() {
            arg_match.values.push_back(arg);
            arg_match.occurs += 1;
        }
        self.arg_index += 1;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ArgMatch {
    values: VecDeque<Value>,
    occurs: usize,
}

#[derive(Default, Debug)]
pub struct Matches {
    args: HashMap<String, ArgMatch>,
}

impl Matches {
    pub fn get(&self, key: &String) -> Option<&ArgMatch> {
        self.args.get(key)
    }

    pub fn conatins(&self, key: &String) -> bool {
        self.get(key).is_some()
    }

    pub fn value(&self, key: &String) -> Option<&Value> {
        self.args.get(key).map(|a| &a.values[0])
    }

    pub fn occurences(&self, key: &String) -> usize {
        self.args.get(key).map(|a| a.occurs).unwrap_or_default()
    }

    pub fn take_value(&mut self, key: &String) -> Option<Value> {
        match self.args.get_mut(key) {
            Some(arg) => arg.values.pop_front(),
            None => None,
        }
    }
}
