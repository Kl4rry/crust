#[derive(PartialEq, Debug, Clone)]
pub struct Span {
    start: u32,
    end: u32,
}

impl Span {
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start as u32,
            end: end as u32,
        }
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start as usize
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end as usize
    }

    #[inline]
    pub fn length(&self) -> usize {
        (self.end - self.start) as usize
    }
}
