use std::{fmt::Debug, num::NonZeroU32};

use crate::{
    lexer::{self, TokenKind},
    span::{ByteSpan, Position, Span, TextSpan},
};
pub use derive_ast_node::{AstNode, MaybeAstNode, SerializeTest};

pub struct Display<'a, T: ?Sized>(&'a T);

impl<T: ?Sized + SerializeTest> core::fmt::Display for Display<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.serialize(f)
    }
}

pub trait SerializeTest {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    fn display_serialize(&self) -> Display<'_, Self>
    where
        Self: Sized,
    {
        Display(self)
    }

    fn to_serialize_string(&self) -> String {
        Display(self).to_string()
    }
}

impl<T: SerializeTest> SerializeTest for [T] {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for a in self {
            a.serialize(f)?;
        }
        Ok(())
    }
}

impl<T: SerializeTest> SerializeTest for Vec<T> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_slice().serialize(f)
    }
}

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

impl<A: ?Sized + SerializeTest> SerializeTest for Box<A> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        A::serialize(self, f)
    }
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct Spanned<T> {
    pub value: Option<T>,
    #[node(spans)]
    pub span: Spans,
}

pub trait Walk {}

#[derive(Debug, Clone, Copy)]
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
    pub ignored: Ignored<'text>,
}

#[derive(Debug)]
pub struct Ignored<'text> {
    inner: Option<Box<IgnoredInner<'text>>>,
}

impl<'text> Ignored<'text> {
    pub(crate) fn new(ignored: Vec<lexer::Token<'text>>, span: Spans) -> Ignored {
        Self {
            inner: if ignored.is_empty() {
                None
            } else {
                Some(Box::new(IgnoredInner {
                    items: ignored,
                    span,
                }))
            },
        }
    }
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct IgnoredInner<'text> {
    pub items: Vec<lexer::Token<'text>>,
    #[node(spans)]
    pub span: Spans,
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

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub enum Stmt<'text> {
    Let(StmtLet<'text>),
    Expr(Expr<'text>),
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub struct StmtLet<'text> {
    #[node(always)]
    pub let_tok: Token![Let<'text>],
    pub name: Ident<'text>,
    pub eq_tok: Token![Eq<'text>],
    #[node(always)]
    pub expr: Expr<'text>,
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct FloatLiteral<'text> {
    #[node(ignore)]
    pub valid: bool,
    #[node(always)]
    pub info: TokenInfo<'text>,
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub enum Expr<'text> {
    IntegerLiteral(Token![Integer<'text>]),
    FloatLiteral(FloatLiteral<'text>),
    Ident(Ident<'text>),
    Grouped(Box<Grouped<'text>>),
    Infix(Box<InfixExpr<'text>>),
    Missing(Spans),
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub struct Grouped<'text> {
    #[node(always)]
    pub open_paren: Token![OpenParen<'text>],
    pub expr: Expr<'text>,
    #[node(always)]
    pub close_paren: Token![CloseParen<'text>],
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub struct InfixExpr<'text> {
    #[node(always)]
    pub left: Expr<'text>,
    pub op: InfixOp<'text>,
    #[node(always)]
    pub right: Expr<'text>,
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub enum InfixOp<'text> {
    Add(Token![Plus<'text>]),
    Sub(Token![Hyphen<'text>]),
    Mul(Token![Star<'text>]),
    Div(Token![ForSlash<'text>]),
}

impl SerializeTest for bool {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt(f)
    }
}

impl<const TOKEN_KIND: u8> Token<'_, TOKEN_KIND> {
    pub const TOKEN_KIND: TokenKind = crate::lexer::TOKEN_KINDS[TOKEN_KIND as usize];
}

impl SerializeTest for Ignored<'_> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(inner) = self.inner.as_ref() {
            inner.serialize(f)?
        }

        Ok(())
    }
}

impl SerializeTest for IgnoredInner<'_> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, ",ignore ")?;
        self.span.serialize(f)?;
        write!(f, " (")?;
        for i in &self.items {
            write!(
                f,
                "{:?} @ {} = {:?},",
                i.kind,
                i.span.display_serialize(),
                i.text
            )?;
        }
        write!(f, "),")?;

        Ok(())
    }
}

impl<const TOKEN_KIND: u8> SerializeTest for Token<'_, TOKEN_KIND> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:?} @ {} / valid={} = {:?}{}",
            Self::TOKEN_KIND,
            self.info.span.display_serialize(),
            self.valid,
            self.info.text,
            self.info.ignored.display_serialize()
        )
    }
}

impl SerializeTest for FloatLiteral<'_> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "FloatLiteral @ {} / valid={} = {:?}{}",
            self.info.span.display_serialize(),
            self.valid,
            self.info.text,
            self.info.ignored.display_serialize()
        )
    }
}

impl SerializeTest for Ident<'_> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:?} <{:?}> @ {}{}",
            self.info.text,
            match &self.id {
                Some(id) => id as &dyn core::fmt::Debug,
                None => &"invalid",
            },
            self.info.span.display_serialize(),
            self.info.ignored.display_serialize()
        )
    }
}

impl SerializeTest for Spans {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} / {:?}", self.byte, self.text)
    }
}

impl SerializeTest for TokenInfo<'_> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:?} @ {}{}",
            self.text,
            self.span.display_serialize(),
            self.ignored.display_serialize()
        )
    }
}
