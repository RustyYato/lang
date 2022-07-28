use std::ops::Index;

use crate::ast;

pub trait Position: Sized {
    fn span(span: &ast::Spans) -> Span<Self>;
    fn start(span: &ast::Spans) -> Self;
    fn end(span: &ast::Spans) -> Self;
}

#[derive(Clone, Copy)]
pub struct BytePos {
    pub pos: u32,
}

impl Position for BytePos {
    fn start(span: &ast::Spans) -> Self {
        span.byte.start
    }

    fn end(span: &ast::Spans) -> Self {
        span.byte.end
    }

    fn span(span: &ast::Spans) -> Span<Self> {
        span.byte
    }
}

#[derive(Clone, Copy)]
pub struct TextPos {
    pub line: u32,
    pub col: u32,
}

impl Position for TextPos {
    fn start(span: &ast::Spans) -> Self {
        span.text.start
    }

    fn end(span: &ast::Spans) -> Self {
        span.text.end
    }

    fn span(span: &ast::Spans) -> Span<Self> {
        span.text
    }
}

pub type ByteSpan = Span<BytePos>;
pub type TextSpan = Span<TextPos>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Span<T> {
    pub start: T,
    pub end: T,
}

impl<T: core::fmt::Debug> core::fmt::Debug for Span<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

impl core::fmt::Debug for BytePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b{}", self.pos)
    }
}

impl core::fmt::Debug for TextPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.col + 1)
    }
}

impl Index<ByteSpan> for str {
    type Output = str;

    fn index(&self, index: ByteSpan) -> &Self::Output {
        &self[index.start.pos as usize..index.end.pos as usize]
    }
}
