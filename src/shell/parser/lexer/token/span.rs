use std::cmp;

use miette::SourceSpan;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Span {
    start: usize,
    len: usize,
}

impl Span {
    #[inline(always)]
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            len: end - start,
        }
    }

    #[inline(always)]
    pub fn start(&self) -> usize {
        self.start as usize
    }

    #[inline(always)]
    pub fn end(&self) -> usize {
        self.start + self.len
    }

    #[inline(always)]
    pub fn length(&self) -> usize {
        self.len
    }
}

impl From<Span> for SourceSpan {
    #[inline(always)]
    fn from(span: Span) -> Self {
        SourceSpan::from((span.start, span.len))
    }
}

impl std::ops::Add for Span {
    type Output = Span;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        let (start1, end1) = (self.start, self.start + self.len);
        let (start2, end2) = (rhs.start, rhs.start + rhs.len);
        let start = cmp::min(start1, start2);
        let end = cmp::max(end1, end2);
        Span::new(start, end)
    }
}
