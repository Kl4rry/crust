pub trait StrExt {
    fn get_char(&self, index: usize) -> Option<char>;
}

impl StrExt for &str {
    fn get_char(&self, index: usize) -> Option<char> {
        self[index..].chars().next()
    }
}
