use std::str::Chars;

use crate::{
    ast,
    span::{BytePos, ByteSpan, TextPos, TextSpan},
};
use unicode_xid::UnicodeXID;

#[derive(Clone, Copy)]
pub struct Pos {
    pub byte: BytePos,
    pub text: TextPos,
}

#[derive(Debug)]
pub struct Token<'text> {
    pub kind: TokenKind,
    pub text: &'text str,
    pub span: ast::Spans,
}

impl Pos {
    pub fn to(self, other: Self) -> ast::Spans {
        ast::Spans {
            byte: ByteSpan {
                start: self.byte,
                end: other.byte,
            },
            text: TextSpan {
                start: self.text,
                end: other.text,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    Eof,
    Unknown,
    WhiteSpace,
    LineComment,

    // symbols
    Plus,
    Hyphen,
    Star,
    ForSlash,
    BackSlash,
    OpenParen,
    CloseParen,
    OpenSquare,
    CloseSquare,
    OpenCurly,
    CloseCurly,
    Dot,

    // keywords
    Match,
    If,
    Else,
    Loop,
    Break,
    Continue,

    // ident + contextual keywords
    BasicIdent,
}

#[derive(Clone)]
pub struct Lexer<'text> {
    text: Chars<'text>,
    pos: u32,
    line: u32,
    col: u32,
}

impl<'text> Lexer<'text> {
    pub fn new(text: &'text str) -> Self {
        Self {
            text: text.chars(),
            pos: 0,
            line: 0,
            col: 0,
        }
    }

    pub fn pos(&self) -> Pos {
        Pos {
            byte: BytePos { pos: self.pos },
            text: TextPos {
                line: self.line,
                col: self.col,
            },
        }
    }

    pub fn lex(&mut self) -> Token<'text> {
        let original = self.text.as_str();
        let start = self.pos();

        let c = match self.text.next() {
            Some(c) => c,
            None => {
                return Token {
                    text: "",
                    span: start.to(start),
                    kind: TokenKind::Eof,
                }
            }
        };

        let mut len = c.len_utf8();
        let mut col_offset = len;
        self.pos += len as u32;

        let mut kind = match c {
            ' ' | '\t' => {
                if let Some(position) = original.bytes().position(|b| !matches!(b, b' ' | b'\t')) {
                    let byte = unsafe { *original.as_bytes().get_unchecked(position) };
                    len = position + usize::from(byte == b'\n');

                    if byte == b'\n' {
                        self.line += 1;
                        self.col = 0;
                        col_offset = 0;
                    } else {
                        col_offset = position;
                    }

                    self.text = unsafe { self.text.as_str().get_unchecked(len - 1..).chars() }
                }

                TokenKind::WhiteSpace
            }
            '\n' => {
                self.line += 1;
                self.col = 0;
                col_offset = 0;
                TokenKind::WhiteSpace
            }
            '+' => TokenKind::Plus,
            '-' => TokenKind::Hyphen,
            '*' => TokenKind::Star,
            '/' => TokenKind::ForSlash,
            '\\' => TokenKind::BackSlash,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '[' => TokenKind::OpenSquare,
            ']' => TokenKind::CloseSquare,
            '{' => TokenKind::OpenCurly,
            '}' => TokenKind::CloseCurly,
            '.' => TokenKind::Dot,
            '#' => TokenKind::LineComment,
            c if c == '_' || c.is_xid_start() => {
                loop {
                    let prev = self.text.clone();
                    if !matches!(self.text.next(), Some(c) if c.is_xid_continue()) {
                        self.text = prev;
                        break;
                    }
                }

                len = original.len() - self.text.as_str().len();

                TokenKind::BasicIdent
            }
            _ => TokenKind::Unknown,
        };

        let text = unsafe { original.get_unchecked(..len) };
        self.col += col_offset as u32;

        if kind == TokenKind::BasicIdent {
            kind = match text {
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "loop" => TokenKind::Loop,
                "break" => TokenKind::Break,
                "continue" => TokenKind::Continue,
                "match" => TokenKind::Match,
                _ => TokenKind::BasicIdent,
            }
        }

        Token {
            text,
            span: start.to(self.pos()),
            kind,
        }
    }
}
