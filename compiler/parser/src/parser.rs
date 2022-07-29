use crate::{
    ast::{self, Spans},
    lexer::{self, Lexer, TokenKind},
    span::{BytePos, ByteSpan, TextPos, TextSpan},
};

use ast::SerializeTest;
use thiserror::Error;

pub struct Parser<'a, 'text> {
    lexer: Lexer<'text>,
    ident_id: u32,
    last_ignore_spans: Spans,
    errors: &'a mut dyn ErrorReporter,
    keep_ignored_tokens: bool,
}

pub trait ErrorReporter {
    fn report(&mut self, error: Error);
}

impl<T: ErrorReporter + ?Sized> ErrorReporter for &mut T {
    fn report(&mut self, error: Error) {
        T::report(self, error)
    }
}

impl ErrorReporter for Vec<Error> {
    fn report(&mut self, error: Error) {
        self.push(error)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Expected expr at {:?}, but found {found:?}", .span.text)]
    ExpectedExpr { found: TokenKind, span: Spans },
    #[error("Expected {expected:?} at {:?}, but found {found:?}", .span.text)]
    UnexpectedToken {
        found: TokenKind,
        expected: TokenKind,
        span: Spans,
    },
}

impl SerializeTest for Error {
    fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error(")?;
        match self {
            Error::ExpectedExpr { found, span } => {
                write!(f, "ExpectedExpr({:?},{},)", found, span.display_serialize())?
            }
            Error::UnexpectedToken {
                expected,
                found,
                span,
            } => write!(
                f,
                "UnexpectedToken({:?},{:?},{},)",
                expected,
                found,
                span.display_serialize()
            )?,
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ExprPrec {
    Expr,

    AddSub,
    MulDiv,
}

enum OpKind {
    Add,
    Sub,
    Mul,
    Div,
}

impl<'a, 'text> Parser<'a, 'text> {
    pub fn new(errors: &'a mut dyn ErrorReporter, text: &'text str) -> Self {
        Self {
            lexer: Lexer::new(text),
            keep_ignored_tokens: true,
            ident_id: 0,
            errors,
            last_ignore_spans: Spans {
                byte: ByteSpan {
                    start: BytePos { pos: 0 },
                    end: BytePos { pos: 0 },
                },
                text: TextSpan {
                    start: TextPos { line: 0, col: 0 },
                    end: TextPos { line: 0, col: 0 },
                },
            },
        }
    }

    pub fn hide_ignored_tokens(&mut self) -> &mut Self {
        self.keep_ignored_tokens = false;
        self
    }

    pub fn peek(&self) -> TokenKind {
        self.peek_token().kind
    }

    pub fn peek_token(&self) -> lexer::Token<'text> {
        self.lexer.clone().lex()
    }

    pub fn next_token(&mut self) -> (TokenKind, ast::TokenInfo<'text>) {
        let token = self.lexer.lex();
        let ignored = self.consume_ignored_tokens();
        (
            token.kind,
            ast::TokenInfo {
                text: token.text,
                span: token.span,
                ignored,
            },
        )
    }

    pub fn consume_ignored_tokens(&mut self) -> ast::Ignored<'text> {
        let start = self.lexer.pos();
        let mut tokens = Vec::new();
        while matches!(self.peek(), TokenKind::WhiteSpace | TokenKind::LineComment) {
            let token = self.lexer.lex();
            if self.keep_ignored_tokens {
                tokens.push(token);
            }
        }
        let end = self.lexer.pos();
        self.last_ignore_spans = start.to(end);
        ast::Ignored::new(tokens, start.to(end))
    }

    pub fn parse_ident(&mut self) -> ast::Ident<'text> {
        let (kind, info) = self.next_token();
        if kind < TokenKind::BasicIdent {
            self.errors.report(Error::UnexpectedToken {
                found: kind,
                expected: TokenKind::BasicIdent,
                span: info.span,
            });
            ast::Ident { id: None, info }
        } else {
            self.ident_id += 1;
            ast::Ident {
                id: Some(ast::IdentId::new(self.ident_id)),
                info,
            }
        }
    }

    pub fn try_parse_ident(&mut self) -> Option<ast::Ident<'text>> {
        if self.peek() < TokenKind::BasicIdent {
            None
        } else {
            let (_, info) = self.next_token();
            self.ident_id += 1;
            Some(ast::Ident {
                id: Some(ast::IdentId::new(self.ident_id)),
                info,
            })
        }
    }

    pub fn parse_token<const TOKEN_KIND: u8>(&mut self) -> ast::Token<'text, TOKEN_KIND> {
        let (kind, info) = self.next_token();
        let valid = kind as u8 == TOKEN_KIND;
        if !valid {
            let expected = ast::Token::<TOKEN_KIND>::TOKEN_KIND;
            self.errors.report(Error::UnexpectedToken {
                found: kind,
                expected,
                span: info.span,
            })
        }
        ast::Token { valid, info }
    }

    pub fn try_parse_token<const TOKEN_KIND: u8>(
        &mut self,
    ) -> Option<ast::Token<'text, TOKEN_KIND>> {
        if self.peek() as u8 == TOKEN_KIND {
            let (_, info) = self.next_token();
            Some(ast::Token { valid: true, info })
        } else {
            None
        }
    }

    pub fn parse_expr(&mut self) -> ast::Expr<'text> {
        self.parse_expr_in(ExprPrec::Expr)
    }

    fn parse_expr_in(&mut self, prec: ExprPrec) -> ast::Expr<'text> {
        let mut expr = self.parse_basic_expr();

        while let Some((op_kind, before, after)) = self.peek_expr_op(prec) {
            if before <= prec {
                break;
            }

            expr = self.finish_expr(expr, op_kind, after)
        }

        expr
    }

    pub fn parse_stmt(&mut self) -> ast::StmtLet<'text> {
        ast::StmtLet {
            let_tok: self.parse_token(),
            name: self.parse_ident(),
            eq_tok: self.parse_token(),
            expr: self.parse_expr(),
        }
    }

    pub fn parse_let_stmt(&mut self) -> ast::StmtLet<'text> {
        ast::StmtLet {
            let_tok: self.parse_token(),
            name: self.parse_ident(),
            eq_tok: self.parse_token(),
            expr: self.parse_expr(),
        }
    }

    fn peek_expr_op(&self, _prec: ExprPrec) -> Option<(OpKind, ExprPrec, ExprPrec)> {
        Some(match self.peek() {
            TokenKind::Plus => (OpKind::Add, ExprPrec::AddSub, ExprPrec::AddSub),
            TokenKind::Hyphen => (OpKind::Sub, ExprPrec::AddSub, ExprPrec::AddSub),
            TokenKind::Star => (OpKind::Mul, ExprPrec::MulDiv, ExprPrec::MulDiv),
            TokenKind::ForSlash => (OpKind::Div, ExprPrec::MulDiv, ExprPrec::MulDiv),
            _ => return None,
        })
    }

    pub fn parse_basic_expr(&mut self) -> ast::Expr<'text> {
        match self.peek() {
            token if token >= TokenKind::BasicIdent => ast::Expr::Ident(self.parse_ident()),

            TokenKind::Integer => ast::Expr::IntegerLiteral(self.parse_token()),
            TokenKind::SimpleFloat => {
                let token: Token![SimpleFloat<'_>] = self.parse_token();
                ast::Expr::FloatLiteral(ast::FloatLiteral {
                    valid: token.valid,
                    info: token.info,
                })
            }
            TokenKind::ExpFloat => {
                let token: Token![ExpFloat<'_>] = self.parse_token();
                ast::Expr::FloatLiteral(ast::FloatLiteral {
                    valid: token.valid,
                    info: token.info,
                })
            }
            TokenKind::SciFloat => {
                let token: Token![SciFloat<'_>] = self.parse_token();
                ast::Expr::FloatLiteral(ast::FloatLiteral {
                    valid: token.valid,
                    info: token.info,
                })
            }

            TokenKind::OpenParen => ast::Expr::Grouped(Box::new(ast::Grouped {
                open_paren: self.parse_token(),
                expr: self.parse_expr(),
                close_paren: self.parse_token(),
            })),

            found @ (TokenKind::Eof
            | TokenKind::Plus
            | TokenKind::Hyphen
            | TokenKind::Star
            | TokenKind::ForSlash) => {
                self.errors.report(Error::ExpectedExpr {
                    found,
                    span: self.last_ignore_spans,
                });
                ast::Expr::Missing(self.last_ignore_spans)
            }

            TokenKind::BasicIdent => unreachable!(),

            TokenKind::Unknown | TokenKind::WhiteSpace | TokenKind::LineComment => unreachable!(),

            found @ (TokenKind::BackSlash
            | TokenKind::CloseParen
            | TokenKind::OpenSquare
            | TokenKind::CloseSquare
            | TokenKind::OpenCurly
            | TokenKind::CloseCurly
            | TokenKind::Dot
            | TokenKind::Eq
            | TokenKind::Let
            | TokenKind::Match
            | TokenKind::If
            | TokenKind::Else
            | TokenKind::Loop
            | TokenKind::Break
            | TokenKind::Continue) => {
                let (_, token) = self.next_token();
                self.errors.report(Error::ExpectedExpr {
                    found,
                    span: token.span,
                });
                ast::Expr::Missing(self.last_ignore_spans)
            }
        }
    }

    fn finish_expr(
        &mut self,
        expr: ast::Expr<'text>,
        op_kind: OpKind,
        prec: ExprPrec,
    ) -> ast::Expr<'text> {
        match op_kind {
            OpKind::Add => {
                let token = self.parse_token();
                ast::Expr::Infix(Box::new(ast::InfixExpr {
                    left: expr,
                    op: ast::InfixOp::Add(token),
                    right: self.parse_expr_in(prec),
                }))
            }
            OpKind::Sub => {
                let token = self.parse_token();
                ast::Expr::Infix(Box::new(ast::InfixExpr {
                    left: expr,
                    op: ast::InfixOp::Sub(token),
                    right: self.parse_expr_in(prec),
                }))
            }
            OpKind::Mul => {
                let token = self.parse_token();
                ast::Expr::Infix(Box::new(ast::InfixExpr {
                    left: expr,
                    op: ast::InfixOp::Mul(token),
                    right: self.parse_expr_in(prec),
                }))
            }
            OpKind::Div => {
                let token = self.parse_token();
                ast::Expr::Infix(Box::new(ast::InfixExpr {
                    left: expr,
                    op: ast::InfixOp::Div(token),
                    right: self.parse_expr_in(prec),
                }))
            }
        }
    }
}
