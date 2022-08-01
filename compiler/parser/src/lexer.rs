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

#[derive(Debug, Clone, Copy)]
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

macro_rules! token_kinds {
    ($($name:ident)*) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum TokenKind {
            $($name,)*
        }

        pub const TOKEN_KINDS: &[TokenKind] = &[$(TokenKind::$name,)*];


    };
}
token_kinds! {
    Eof
    Unknown
    WhiteSpace
    LineComment

    // symbols
    Plus
    Hyphen
    Star
    ForSlash
    BackSlash
    OpenParen
    CloseParen
    OpenSquare
    CloseSquare
    OpenCurly
    CloseCurly
    Dot
    Eq
    Semicolon

    // no period or exp
    Integer
    // only period
    SimpleFloat
    // only exp
    ExpFloat
    // period and exp
    SciFloat

    // keywords
    Let
    Match
    While
    If
    Else
    Loop
    Break
    Continue

    // ident + contextual keywords
    BasicIdent
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
        let mut col_offset = None;

        let mut kind = match c {
            ' ' | '\t' => {
                if let Some(position) = original.bytes().position(|b| !matches!(b, b' ' | b'\t')) {
                    let byte = unsafe { *original.as_bytes().get_unchecked(position) };
                    len = position + usize::from(byte == b'\n');

                    if byte == b'\n' {
                        self.line += 1;
                        self.col = 0;
                        col_offset = Some(0);
                    }

                    self.text = unsafe { self.text.as_str().get_unchecked(len - 1..).chars() }
                }

                TokenKind::WhiteSpace
            }
            '\n' => {
                self.line += 1;
                self.col = 0;
                col_offset = Some(0);
                TokenKind::WhiteSpace
            }
            '=' => TokenKind::Eq,
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
            ';' => TokenKind::Semicolon,
            '#' => {
                len = memchr::memchr(b'\n', original.as_bytes()).unwrap_or(original.len());
                self.text = original[len..].chars();
                TokenKind::LineComment
            }
            '0'..='9' => {
                let bytes = original.as_bytes();
                let end = bytes
                    .iter()
                    .position(|&byte| !matches!(byte, b'0'..=b'9'))
                    .unwrap_or(bytes.len());

                let bytes = &bytes[end..];

                let mut has_period = false;
                let mut has_exp = false;

                let bytes = if let [b'.', b'0'..=b'9', bytes @ ..] = bytes {
                    has_period = true;
                    let end = bytes
                        .iter()
                        .position(|&byte| !matches!(byte, b'0'..=b'9'))
                        .unwrap_or(bytes.len());
                    &bytes[end..]
                } else {
                    bytes
                };

                let bytes = if let [b'e', b'+' | b'-', b'0'..=b'9', bytes @ ..]
                | [b'e', b'0'..=b'9', bytes @ ..] = bytes
                {
                    has_exp = true;
                    let end = bytes
                        .iter()
                        .position(|&byte| !matches!(byte, b'0'..=b'9'))
                        .unwrap_or(bytes.len());
                    &bytes[end..]
                } else {
                    bytes
                };

                let text = original;
                len = text.len() - bytes.len();
                self.text = text[text.len() - bytes.len()..].chars();

                match (has_period, has_exp) {
                    (true, true) => TokenKind::SciFloat,
                    (true, false) => TokenKind::SimpleFloat,
                    (false, true) => TokenKind::ExpFloat,
                    (false, false) => TokenKind::Integer,
                }
            }
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
        self.pos += len as u32;
        self.col += col_offset.unwrap_or(len) as u32;

        if kind == TokenKind::BasicIdent {
            kind = match text {
                "let" => TokenKind::Let,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "loop" => TokenKind::Loop,
                "break" => TokenKind::Break,
                "while" => TokenKind::While,
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

    pub(crate) fn is_eof(&self) -> bool {
        self.text.as_str() == ""
    }
}
