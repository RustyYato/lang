use std::{
    fmt::Debug,
    num::NonZeroU32,
    ops::{Index, Range, RangeInclusive},
};

use crate::{
    lexer::TokenKind,
    span::{ByteSpan, TextSpan},
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
    fn span(&self) -> RangeInclusive<TokenId> {
        self.start()..=self.end()
    }

    fn start(&self) -> TokenId;

    fn end(&self) -> TokenId;
}

pub trait MaybeAstNode {
    fn try_span(&self) -> Option<RangeInclusive<TokenId>> {
        Some(self.try_start()?..=self.try_end()?)
    }

    fn try_start(&self) -> Option<TokenId>;

    fn try_end(&self) -> Option<TokenId>;
}

impl<A: ?Sized + MaybeAstNode> MaybeAstNode for Box<A> {
    fn try_span(&self) -> Option<RangeInclusive<TokenId>> {
        A::try_span(self)
    }

    fn try_start(&self) -> Option<TokenId> {
        A::try_start(self)
    }

    fn try_end(&self) -> Option<TokenId> {
        A::try_end(self)
    }
}

impl<A: ?Sized + AstNode> AstNode for Box<A> {
    fn span(&self) -> RangeInclusive<TokenId> {
        A::span(self)
    }

    fn start(&self) -> TokenId {
        A::start(self)
    }

    fn end(&self) -> TokenId {
        A::end(self)
    }
}

impl<A: ?Sized + SerializeTest> SerializeTest for Box<A> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        A::serialize(self, f)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Spans {
    pub byte: ByteSpan,
    pub text: TextSpan,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenId(NonZeroU32);

impl core::fmt::Debug for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tok.{}", self.0)
    }
}

impl MaybeAstNode for TokenId {
    fn try_span(&self) -> Option<RangeInclusive<TokenId>> {
        Some(Self::span(self))
    }

    fn try_start(&self) -> Option<TokenId> {
        Some(Self::start(self))
    }

    fn try_end(&self) -> Option<TokenId> {
        Some(Self::end(self))
    }
}

impl AstNode for TokenId {
    fn start(&self) -> TokenId {
        *self
    }

    fn end(&self) -> TokenId {
        *self
    }
}

impl TokenId {
    fn get(self) -> usize {
        self.0.get().wrapping_sub(1) as usize
    }
}

pub struct TokenList<'text> {
    items: Vec<TokenInfo<'text>>,
    ignored: Option<Vec<Ignored>>,
}

impl<'text> TokenList<'text> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            ignored: Some(Vec::new()),
        }
    }

    pub fn new_drop_ignored() -> Self {
        Self {
            items: Vec::new(),
            ignored: None,
        }
    }

    fn new_token_id(&self) -> TokenId {
        TokenId(
            NonZeroU32::new(self.items.len() as u32).expect("cannot create more tham u32::MAX ids"),
        )
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push_ignored(&mut self, token: TokenInfo<'text>) {
        if self.ignored.is_some() {
            self.items.push(token);
            self.new_token_id();
        }
    }

    pub fn push(&mut self, token: TokenInfo<'text>, ignored: Ignored) -> TokenId {
        self.items.push(token);
        if let Some(ignored_items) = self.ignored.as_mut() {
            ignored_items.push(ignored);
        }
        self.new_token_id()
    }
}

impl<'text> Index<TokenId> for TokenList<'text> {
    type Output = TokenInfo<'text>;

    fn index(&self, index: TokenId) -> &Self::Output {
        &self.items[index.get()]
    }
}

impl<'text> Index<core::ops::RangeInclusive<TokenId>> for TokenList<'text> {
    type Output = [TokenInfo<'text>];

    fn index(&self, index: core::ops::RangeInclusive<TokenId>) -> &Self::Output {
        &self.items[index.start().get()..=index.end().get()]
    }
}

#[derive(Debug)]
pub struct TokenInfo<'text> {
    pub kind: TokenKind,
    pub text: &'text str,
    pub span: Spans,
}

#[derive(Debug, Clone)]
pub struct Ignored {
    pub items: Range<usize>,
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
pub struct Ident {
    #[node(ignore)]
    pub name: ustr::Ustr,
    #[node(ignore)]
    pub id: Option<IdentId>,
    #[node(always)]
    pub tok_id: TokenId,
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct Token<const TOKEN_KIND: u8> {
    #[node(ignore)]
    pub valid: bool,
    #[node(always)]
    pub tok_id: TokenId,
}

macro_rules! Token {
    (Ident) => {
        $crate::ast::Ident
    };
    ($name:ident) => {
        $crate::ast::Token<{$crate::lexer::TokenKind::$name as u8}>
    };
}

#[derive(Debug, MaybeAstNode, SerializeTest)]
pub enum Stmt {
    Let(StmtLet),
    Expr(Expr),
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub struct StmtLet {
    #[node(always)]
    pub let_tok: Token![Let],
    pub name: Ident,
    #[node(always)]
    pub eq_tok: Token![Eq],
    pub expr: Expr,
}

#[derive(Debug, MaybeAstNode, AstNode)]
pub struct FloatLiteral {
    #[node(ignore)]
    pub valid: bool,
    #[node(always)]
    pub tok_id: TokenId,
}

#[derive(Debug, MaybeAstNode, SerializeTest)]
pub enum Expr {
    IntegerLiteral(Token![Integer]),
    FloatLiteral(FloatLiteral),
    Ident(Ident),
    Grouped(Box<Grouped>),
    Infix(Box<InfixExpr>),
    Missing(MissingExpr),
}

#[derive(Debug, SerializeTest)]
pub struct MissingExpr;

impl MaybeAstNode for MissingExpr {
    fn try_start(&self) -> Option<TokenId> {
        None
    }

    fn try_end(&self) -> Option<TokenId> {
        None
    }
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub struct Grouped {
    #[node(always)]
    pub open_paren: Token![OpenParen],
    pub expr: Expr,
    #[node(always)]
    pub close_paren: Token![CloseParen],
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub struct InfixExpr {
    pub left: Expr,
    #[node(always)]
    pub op: InfixOp,
    pub right: Expr,
}

#[derive(Debug, MaybeAstNode, AstNode, SerializeTest)]
pub enum InfixOp {
    Add(Token![Plus]),
    Sub(Token![Hyphen]),
    Mul(Token![Star]),
    Div(Token![ForSlash]),
}

impl SerializeTest for bool {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt(f)
    }
}

impl<const TOKEN_KIND: u8> Token<TOKEN_KIND> {
    pub const TOKEN_KIND: TokenKind = crate::lexer::TOKEN_KINDS[TOKEN_KIND as usize];
}

impl<const TOKEN_KIND: u8> SerializeTest for Token<TOKEN_KIND> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:?} {:?} valid={}",
            Self::TOKEN_KIND,
            self.tok_id,
            self.valid
        )
    }
}

impl SerializeTest for FloatLiteral {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FloatLiteral {:?} valid={}", self.tok_id, self.valid)
    }
}

impl SerializeTest for Ident {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Ident {} {:?} {:?}",
            self.name,
            match &self.id {
                Some(id) => id as &dyn core::fmt::Debug,
                None => &"invalid",
            },
            self.tok_id
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
        write!(f, "{:?} @ {} = {:?}", self.kind, self.span.display_serialize(), self.text)
    }
}

impl SerializeTest for TokenList<'_> {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TokenList(")?;
        write!(f, "Tokens(")?;
        for item in &self.items {
            item.serialize(f)?;
            write!(f, ",")?;
        }
        write!(f, ")")?;

        if let Some(ignored) = &self.ignored {
            write!(f, "Ignored(")?;
            for item in ignored {
                write!(f, "{:?},", item.items)?;
            }
            write!(f, ")")?;
        }

        write!(f, ")")
    }
}

impl<A: SerializeTest, B: SerializeTest> SerializeTest for (A, B) {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.serialize(f)?;
        self.1.serialize(f)
    }
}
