use crate::{
    ast::{self, Spans, TokenId, TokenList},
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
    token_list: TokenList<'text>,
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
            ident_id: 0,
            errors,
            token_list: TokenList::new(),
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

    pub fn finish(self) -> TokenList<'text> {
        self.token_list
    }

    pub fn hide_ignored_tokens(&mut self) -> &mut Self {
        assert!(self.token_list.is_empty());
        self.token_list = TokenList::new_drop_ignored();
        self
    }

    pub fn peek(&self) -> TokenKind {
        self.peek_token().kind
    }

    pub fn peek_token(&self) -> lexer::Token<'text> {
        self.lexer.clone().lex()
    }

    pub fn next_token(&mut self) -> (TokenKind, TokenId) {
        let token = self.lexer.lex();
        let ignored = self.consume_ignored_tokens();
        let tok_id = self.token_list.push(
            ast::TokenInfo {
                kind: token.kind,
                text: token.text,
                span: token.span,
            },
            ignored,
        );
        (token.kind, tok_id)
    }

    pub fn consume_ignored_tokens(&mut self) -> ast::Ignored {
        let start = self.lexer.pos();
        let tok_start = self.token_list.len();
        loop {
            let lexer = self.lexer.clone();
            let token = self.lexer.lex();

            if matches!(token.kind, TokenKind::WhiteSpace | TokenKind::LineComment) {
                self.token_list.push_ignored(ast::TokenInfo {
                    kind: token.kind,
                    text: token.text,
                    span: token.span,
                })
            } else {
                self.lexer = lexer;
                break;
            }
        }
        let end = self.lexer.pos();
        self.last_ignore_spans = start.to(end);
        let tok_end = self.token_list.len();
        ast::Ignored {
            items: tok_start..tok_end,
        }
    }

    pub fn parse_ident(&mut self) -> ast::Ident {
        let (kind, tok_id) = self.next_token();
        let name = ustr::ustr(self.token_list[tok_id].text);
        if kind < TokenKind::BasicIdent {
            self.errors.report(Error::UnexpectedToken {
                found: kind,
                expected: TokenKind::BasicIdent,
                span: self.token_list[tok_id].span,
            });
            ast::Ident {
                name,
                id: None,
                tok_id,
            }
        } else {
            self.ident_id += 1;
            ast::Ident {
                name,
                id: Some(ast::IdentId::new(self.ident_id)),
                tok_id,
            }
        }
    }

    pub fn try_parse_ident(&mut self) -> Option<ast::Ident> {
        if self.peek() < TokenKind::BasicIdent {
            None
        } else {
            let (_, tok_id) = self.next_token();
            let name = ustr::ustr(self.token_list[tok_id].text);
            self.ident_id += 1;
            Some(ast::Ident {
                name,
                id: Some(ast::IdentId::new(self.ident_id)),
                tok_id,
            })
        }
    }

    pub fn parse_token<const TOKEN_KIND: u8>(&mut self) -> ast::Token<TOKEN_KIND> {
        let (kind, tok_id) = self.next_token();
        let valid = kind as u8 == TOKEN_KIND;
        if !valid {
            let expected = ast::Token::<TOKEN_KIND>::TOKEN_KIND;
            self.errors.report(Error::UnexpectedToken {
                found: kind,
                expected,
                span: self.token_list[tok_id].span,
            })
        }
        ast::Token { valid, tok_id }
    }

    pub fn try_parse_token<const TOKEN_KIND: u8>(&mut self) -> Option<ast::Token<TOKEN_KIND>> {
        if self.peek() as u8 == TOKEN_KIND {
            let (_, tok_id) = self.next_token();
            Some(ast::Token {
                valid: true,
                tok_id,
            })
        } else {
            None
        }
    }

    pub fn parse_expr(&mut self) -> ast::Expr {
        self.parse_expr_in(ExprPrec::Expr)
    }

    fn parse_expr_in(&mut self, prec: ExprPrec) -> ast::Expr {
        let mut expr = self.parse_basic_expr();

        while let Some((op_kind, before, after)) = self.peek_expr_op(prec) {
            if before <= prec {
                break;
            }

            expr = self.finish_expr(expr, op_kind, after)
        }

        expr
    }

    pub fn parse_stmt(&mut self) -> ast::StmtLet {
        ast::StmtLet {
            let_tok: self.parse_token(),
            name: self.parse_ident(),
            eq_tok: self.parse_token(),
            expr: self.parse_expr(),
        }
    }

    pub fn parse_let_stmt(&mut self) -> ast::StmtLet {
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

    pub fn parse_basic_expr(&mut self) -> ast::Expr {
        match self.peek() {
            token if token >= TokenKind::BasicIdent => ast::Expr::Ident(self.parse_ident()),

            TokenKind::Integer => ast::Expr::IntegerLiteral(self.parse_token()),
            TokenKind::SimpleFloat => {
                let token: Token![SimpleFloat] = self.parse_token();
                ast::Expr::FloatLiteral(ast::FloatLiteral {
                    valid: token.valid,
                    tok_id: token.tok_id,
                })
            }
            TokenKind::ExpFloat => {
                let token: Token![ExpFloat] = self.parse_token();
                ast::Expr::FloatLiteral(ast::FloatLiteral {
                    valid: token.valid,
                    tok_id: token.tok_id,
                })
            }
            TokenKind::SciFloat => {
                let token: Token![SciFloat] = self.parse_token();
                ast::Expr::FloatLiteral(ast::FloatLiteral {
                    valid: token.valid,
                    tok_id: token.tok_id,
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
                ast::Expr::Missing(ast::MissingExpr)
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
                    span: self.token_list[token].span,
                });
                ast::Expr::Missing(ast::MissingExpr)
            }
        }
    }

    fn finish_expr(&mut self, expr: ast::Expr, op_kind: OpKind, prec: ExprPrec) -> ast::Expr {
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
