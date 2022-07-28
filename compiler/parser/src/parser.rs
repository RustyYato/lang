use crate::{
    ast,
    lexer::{Lexer, TokenKind},
};

pub struct Parser<'text> {
    lexer: Lexer<'text>,
    ident_id: u32,
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
        self.consume_ignored_tokens();
        (
            token.kind,
            ast::TokenInfo {
                text: token.text,
                span: token.span,
            },
        )
    }

    pub fn consume_ignored_tokens(&mut self) {
        self.lexer.pos();
        while matches!(self.peek(), TokenKind::WhiteSpace | TokenKind::LineComment) {
            self.lexer.lex();
        }
    }

    pub fn parse_ident(&mut self) -> ast::Ident<'text> {
        let (kind, token) = self.next_token();
        if kind < TokenKind::BasicIdent {
            ast::Ident {
                id: None,
                text: token.text,
                span: token.span,
            }
        } else {
            self.ident_id += 1;
            ast::Ident {
                id: Some(ast::IdentId::new(self.ident_id)),
                text: token.text,
                span: token.span,
            }
        }
    }

    pub fn try_parse_ident(&mut self) -> Option<ast::Ident<'text>> {
        if self.peek() < TokenKind::BasicIdent {
            None
        } else {
            let (_, token) = self.next_token();
            self.ident_id += 1;
            Some(ast::Ident {
                id: Some(ast::IdentId::new(self.ident_id)),
                text: token.text,
                span: token.span,
            })
        }
    }

    pub fn parse_token<const TOKEN_KIND: u8>(&mut self) -> ast::Token<'text, TOKEN_KIND> {
        let (kind, token) = self.next_token();
        ast::Token {
            valid: kind as u8 == TOKEN_KIND,
            text: token.text,
            span: token.span,
        }
    }

    pub fn try_parse_token<const TOKEN_KIND: u8>(
        &mut self,
    ) -> Option<ast::Token<'text, TOKEN_KIND>> {
        if self.peek() as u8 == TOKEN_KIND {
            let (_, token) = self.next_token();
            Some(ast::Token {
                valid: true,
                text: token.text,
                span: token.span,
            })
        } else {
            None
        }
    }
}
