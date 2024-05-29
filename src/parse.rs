//! Parse a stream of `Tokens` into an *Abstract Syntax Tree* we can use for
//! the later steps.
#![allow(missing_docs, dead_code, unused_imports)]

pub use self::ast::{DottedIdent, Ident, Literal, LiteralKind};
pub use self::parser::Parser;

#[macro_use]
mod macros;
mod parser;
mod ast;
