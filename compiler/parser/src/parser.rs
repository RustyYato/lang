use crate::{
    ast,
    lexer::{self, Lexer, TokenKind},
};

pub struct Parser<'text> {
    lexer: Lexer<'text>,
    ident_id: u32,
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

impl<'text> Parser<'text> {
    pub fn new(text: &'text str) -> Self {
        Self {
            lexer: Lexer::new(text),
            ident_id: 0,
        }
    }

    pub fn peek(&self) -> TokenKind {
        self.lexer.clone().lex().kind
    }

    pub fn next_token(&mut self) -> (TokenKind, ast::TokenInfo<'text>) {
        let token = self.lexer.lex();
        let start = self.lexer.pos();
        let ignored = self.consume_ignored_tokens();
        let end = self.lexer.pos();
        (
            token.kind,
            ast::TokenInfo {
                text: token.text,
                span: token.span,
                ignored,
                ignored_span: start.to(end),
            },
        )
    }

    pub fn consume_ignored_tokens(&mut self) -> Vec<lexer::Token<'text>> {
        let tokens = Vec::new();
        while matches!(self.peek(), TokenKind::WhiteSpace | TokenKind::LineComment) {
            self.lexer.lex();
        }
        tokens
    }

    pub fn parse_ident(&mut self) -> ast::Ident<'text> {
        let (kind, info) = self.next_token();
        if kind < TokenKind::BasicIdent {
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
        ast::Token {
            valid: kind as u8 == TOKEN_KIND,
            info,
        }
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

    fn peek_expr_op(&self, prec: ExprPrec) -> Option<(OpKind, ExprPrec, ExprPrec)> {
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

            TokenKind::BasicIdent => unreachable!(),

            TokenKind::Eof => todo!(),
            TokenKind::Unknown => todo!(),
            TokenKind::WhiteSpace => todo!(),
            TokenKind::LineComment => todo!(),
            TokenKind::Plus => todo!(),
            TokenKind::Hyphen => todo!(),
            TokenKind::Star => todo!(),
            TokenKind::ForSlash => todo!(),
            TokenKind::BackSlash => todo!(),
            TokenKind::OpenParen => todo!(),
            TokenKind::CloseParen => todo!(),
            TokenKind::OpenSquare => todo!(),
            TokenKind::CloseSquare => todo!(),
            TokenKind::OpenCurly => todo!(),
            TokenKind::CloseCurly => todo!(),
            TokenKind::Dot => todo!(),
            TokenKind::Match => todo!(),
            TokenKind::If => todo!(),
            TokenKind::Else => todo!(),
            TokenKind::Loop => todo!(),
            TokenKind::Break => todo!(),
            TokenKind::Continue => todo!(),
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
