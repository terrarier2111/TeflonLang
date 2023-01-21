use std::error::Error;
use std::fmt::{Arguments, Debug, Display, Formatter, Write};

#[derive(Debug, Copy, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize, // FIXME: should this be inclusive or exclusive?
}

impl Span {
    pub const NONE: Span = Span {
        start: usize::MAX,
        end: usize::MAX,
    };

    pub const fn single_token(position: usize) -> Self {
        Self {
            start: position,
            end: position + 1, // FIXME: should this be `position`?
        }
    }

    #[inline]
    pub fn multi_token(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn shrink_hi(&mut self) -> Result<(), ShrinkHiError> {
        if self.start == self.end {
            return Err(ShrinkHiError(self.start));
        }
        self.end = self.end - 1;
        Ok(())
    }

    pub fn shrink_lo(&mut self) -> Result<(), ShrinkLoError> {
        if self.start == self.end {
            return Err(ShrinkLoError(self.end));
        }
        self.end = self.end - 1;
        Ok(())
    }

    pub fn is_none(&self) -> bool {
        // we assume spans that are starting at usize::MAX - u8::MAX are also none spans
        // because we need this condition in order to have safe NONE FixedTokenSpans
        // and the likelihood of a valid span being considered invalid because of this is close to zero
        self.start >= (Self::NONE.start - u8::MAX as usize) && self.end == Self::NONE.end
    }
}

impl GenericSpan for Span {
    #[inline(always)]
    fn start(&self) -> usize {
        self.start
    }

    #[inline(always)]
    fn end(&self) -> usize {
        self.end
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FixedTokenSpan<const SIZE: usize = 1>(pub usize); // FIXME: make use of FixedTokenSpan

impl<const SIZE: usize> FixedTokenSpan<SIZE> {

    pub const NONE: FixedTokenSpan<1> = FixedTokenSpan::new(usize::MAX - 1);

    #[inline(always)]
    pub const fn new(start: usize) -> Self {
        Self(start)
    }

    pub fn to_unfixed_span(self) -> Span {
        Span::multi_token(self.start(), self.end())
    }
}

impl<const SIZE: usize> GenericSpan for FixedTokenSpan<SIZE> {
    #[inline(always)]
    fn start(&self) -> usize {
        self.0
    }

    #[inline]
    fn end(&self) -> usize {
        self.0 + SIZE // FIXME: should this be "self.0 - 1 + SIZE"?
    }
}

impl<const SIZE: usize> From<usize> for FixedTokenSpan<SIZE> {
    #[inline(always)]
    fn from(start: usize) -> Self {
        Self::new(start)
    }
}

pub trait GenericSpan {
    fn start(&self) -> usize;

    fn end(&self) -> usize;
}

pub struct ShrinkHiError(usize);

impl Debug for ShrinkHiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("tried to shrink a span starting at ")?;
        let start = self.0.to_string();
        let start = start.as_str();
        f.write_str(start)?;
        f.write_str(" below its start")
    }
}

impl Display for ShrinkHiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dbg = format!("{:?}", self);
        f.write_str(dbg.as_str())
    }
}

impl Error for ShrinkHiError {}

pub struct ShrinkLoError(usize);

impl Debug for ShrinkLoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("tried to shrink a span ending at ")?;
        let end = self.0.to_string();
        let end = end.as_str();
        f.write_str(end)?;
        f.write_str(" above its end")
    }
}

impl Display for ShrinkLoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dbg = format!("{:?}", self);
        f.write_str(dbg.as_str())
    }
}

impl Error for ShrinkLoError {}
