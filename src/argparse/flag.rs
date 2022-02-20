#[derive(Debug)]
pub struct Flag {
    pub(super) name: String,
    pub(super) help: String,
    pub(super) long: Option<String>,
    pub(super) short: Option<char>,
    pub(super) multiple: bool,
}

impl Flag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: String::new(),
            long: None,
            short: None,
            multiple: false,
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
}
