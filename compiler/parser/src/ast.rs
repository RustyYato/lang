use std::num::NonZeroU32;

use crate::{
    lexer,
    span::{ByteSpan, Position, Span, TextSpan},
};
use derive_ast_node::{AstNode, MaybeAstNode};

pub trait AstNode: MaybeAstNode {
    fn span<T: Position>(&self) -> Span<T> {
        Span {
            start: self.start(),
            end: self.end(),
        }
    }

    fn start<T: Position>(&self) -> T;

    fn end<T: Position>(&self) -> T;
}

pub trait MaybeAstNode {
    fn try_span<T: Position>(&self) -> Option<Span<T>> {
        Some(Span {
            start: self.try_start()?,
            end: self.try_end()?,
        })
    }

    fn try_start<T: Position>(&self) -> Option<T>;

    fn try_end<T: Position>(&self) -> Option<T>;
}

impl<A: ?Sized + MaybeAstNode> MaybeAstNode for Box<A> {
    fn try_span<T: Position>(&self) -> Option<Span<T>> {
        A::try_span(self)
    }

    fn try_start<T: Position>(&self) -> Option<T> {
        A::try_start(self)
    }

    fn try_end<T: Position>(&self) -> Option<T> {
        A::try_end(self)
    }
}

impl<A: ?Sized + AstNode> AstNode for Box<A> {
    fn span<T: Position>(&self) -> Span<T> {
        A::span(self)
    }

    fn start<T: Position>(&self) -> T {
        A::start(self)
    }

    fn end<T: Position>(&self) -> T {
        A::end(self)
    }
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct Spanned<T> {
    pub value: Option<T>,
    #[node(spans)]
    pub span: Spans,
}

pub trait Walk {}

#[derive(Debug)]
pub struct Spans {
    pub byte: ByteSpan,
    pub text: TextSpan,
}

impl MaybeAstNode for Spans {
    fn try_start<T: Position>(&self) -> Option<T> {
        Some(T::start(self))
    }

    fn try_end<T: Position>(&self) -> Option<T> {
        Some(T::end(self))
    }

    fn try_span<T: Position>(&self) -> Option<Span<T>> {
        Some(T::span(self))
    }
}

impl AstNode for Spans {
    fn start<T: Position>(&self) -> T {
        T::start(self)
    }

    fn end<T: Position>(&self) -> T {
        T::end(self)
    }

    fn span<T: Position>(&self) -> Span<T> {
        T::span(self)
    }
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct TokenInfo<'text> {
    pub text: &'text str,
    #[node(spans)]
    pub span: Spans,
    pub ignored: Vec<lexer::Token<'text>>,
    pub ignored_span: Spans,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentId(NonZeroU32);
impl core::fmt::Debug for IdentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ident.{}", self.0)
    }
}

impl IdentId {
    pub fn new(id: u32) -> Self {
        Self(NonZeroU32::new(id).expect("cannot have more than u32::MAX identifiers because there are less than u32::MAX bytes in the input"))
    }
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct Ident<'text> {
    #[node(ignore)]
    pub id: Option<IdentId>,
    #[node(always)]
    pub info: TokenInfo<'text>,
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct Token<'text, const TOKEN_KIND: u8> {
    #[node(ignore)]
    pub valid: bool,
    #[node(always)]
    pub info: TokenInfo<'text>,
}

macro_rules! Token {
    (Ident<$lt:lifetime>) => {
        $crate::ast::Ident<$lt>
    };
    ($name:ident<$lt:lifetime>) => {
        $crate::ast::Token<$lt, {$crate::lexer::TokenKind::$name as u8}>
    };
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub enum Expr<'text> {
    IntegerLiteral(TokenInfo<'text>),
    Ident(Ident<'text>),
    Infix(Box<InfixExpr<'text>>),
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct InfixExpr<'text> {
    #[node(always)]
    pub left: Expr<'text>,
    pub op: InfixOp<'text>,
    #[node(always)]
    pub right: Expr<'text>,
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub enum InfixOp<'text> {
    Add(Token![Plus<'text>]),
    Sub(Token![Hyphen<'text>]),
    Mul(Token![Star<'text>]),
    Div(Token![ForSlash<'text>]),
}
