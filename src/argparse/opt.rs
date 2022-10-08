use std::fmt;

use super::Type;

#[derive(Debug)]
pub struct Opt {
    pub(super) name: String,
    pub(super) help: String,
    pub(super) long: Option<String>,
    pub(super) short: Option<char>,
    pub(super) value: Type,
    pub(super) required: bool,
    pub(super) multiple: bool,
    pub(super) conflicts: Vec<String>,
}

impl Opt {
    pub fn new(name: impl Into<String>, value: Type) -> Self {
        Self {
            name: name.into(),
            help: String::new(),
            long: None,
            short: None,
            value,
            required: false,
            multiple: false,
            conflicts: Vec::new(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name.clear();
        self.name.push_str(name);
        self
    }

    pub fn help(mut self, about: &str) -> Self {
        self.help.clear();
        self.help.push_str(about);
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn long(mut self, long: &str) -> Self {
        self.long = Some(long.trim_end_matches(|c| c == '-').to_string());
        self
    }

    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    pub fn conflicts_with(mut self, conflict: String) -> Self {
        self.conflicts.push(conflict);
        self
    }
}

impl fmt::Display for Opt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.long {
            Some(long) => write!(f, "--{}", long)?,
            None => write!(f, "-{}", self.short.unwrap())?,
        }
        write!(f, " <{}>", self.name)?;
        if self.multiple {
            write!(f, "...")?;
        }
        Ok(())
    }
}
