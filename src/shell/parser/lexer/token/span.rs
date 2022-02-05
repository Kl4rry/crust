use std::cmp;

use miette::SourceSpan;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Span {
    start: usize,
    len: usize,
}

impl Span {
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start,
            len: end - start,
        }
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start as usize
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.start + self.len
    }

    #[inline]
    pub fn length(&self) -> usize {
        self.len
    }
}

impl Into<SourceSpan> for Span {
    fn into(self) -> SourceSpan {
        SourceSpan::from((self.start, self.len))
    }
}

impl std::ops::Add for Span {
    type Output = Span;
    fn add(self, rhs: Self) -> Self::Output {
        let (start1, end1) = (self.start, self.start + self.len);
        let (start2, end2) = (rhs.start, rhs.start + rhs.len);
        let start = cmp::min(start1, start2);
        let end = cmp::max(end1, end2);
        Span::new(start, end)
    }
}
