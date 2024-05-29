use std::rc::Rc;

use crate::codemap::{FileMap, Span};
use crate::errors::*;
use crate::lex::{Token, TokenKind};
use crate::parse::ast::{DottedIdent, Ident, Literal, LiteralKind};

/// A parser for turning a stream of tokens into a Abstract Syntax Tree.
#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    filemap: Rc<FileMap>,
    current_index: usize,
}

impl Parser {
    /// Create a new parser.
    pub fn new(tokens: Vec<Token>, filemap: Rc<FileMap>) -> Parser {
        let current_index = 0;
        Parser { tokens, filemap, current_index }
    }

    /// Peek at the current token.
    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.current_index).map(|t| &t.kind)
    }

    /// Get the current token, moving the index along one.
    fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.current_index);

        if tok.is_some() {
            self.current_index += 1;
        }

        tok
    }
}

impl Parser {
    fn parse_literal(&mut self) -> Result<Literal> {
        match self.peek() {
            Some(&TokenKind::Integer(_)) |
            Some(&TokenKind::Decimal(_)) |
            Some(&TokenKind::QuotedString(_)) => {}
            Some(_) => bail!("Expected a literal"),
            None => bail!(ErrorKind::UnexpectedEOF),
        };

        let next = self.next().expect("unreachable");
        let lit_kind = match next.kind {
            TokenKind::Integer(i) => LiteralKind::Integer(i),
            TokenKind::Decimal(d) => LiteralKind::Decimal(d),
            TokenKind::QuotedString(ref s) => LiteralKind::String(s.clone()),
            ref other => panic!("Unreachable token kind: {:?}", other),
        };

        Ok(Literal {
            span: next.span,
            kind: lit_kind,
        })
    }
}

impl Parser {
    fn parse_ident(&mut self) -> Result<Ident> {
        match self.peek() {
            Some(&TokenKind::Identifier(_)) => {}
            _ => bail!("Expected an identifier"),
        }

        let next = self.next().unwrap();

        if let TokenKind::Identifier(ref ident) = next.kind {
            Ok(Ident {
                span: next.span,
                name: ident.clone(),
            })
        } else {
            unreachable!()
        }
    }

    fn parse_dotted_ident(&mut self) -> Result<DottedIdent> {
        let first = self.parse_ident()?;
        let mut parts = vec![first];

        while self.peek() == Some(&TokenKind::Dot) {
            let _ = self.next();
            let next = self.parse_ident()?;
            parts.push(next);
        }

        // the span for a dotted ident should be the union of the spans for
        // each of its components.
        let span = parts.iter()
            .skip(1)
            .fold(parts[0].span, |l, r| self.filemap.merge(l, r.span));

        Ok(DottedIdent { span, parts })
    }
}