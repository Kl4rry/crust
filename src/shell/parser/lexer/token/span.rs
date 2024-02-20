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
        debug_assert!(start <= end);
        Self {
            start,
            len: end - start,
        }
    }

    #[inline(always)]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline(always)]
    pub fn end(&self) -> usize {
        self.start + self.len
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
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
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign for Span {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        let (start1, end1) = (self.start, self.start + self.len);
        let (start2, end2) = (rhs.start, rhs.start + rhs.len);
        self.start = cmp::min(start1, start2);
        self.len = cmp::max(end1, end2) - self.start;
    }
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }
}
