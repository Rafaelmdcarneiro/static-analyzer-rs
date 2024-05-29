//! A parser and static analysis library for exploring Delphi code.
//!
//! This is written using a *Literate Programming* style, so you may find it
//! easier to inspect the [rendered version] instead.
//!
//! [rendered version]: https://michael-f-bryan.github.io/static-analyser-in-rust/

#![deny(missing_docs)]

#[macro_use]
extern crate error_chain;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

extern crate serde;

#[macro_use]
extern crate serde_derive;

pub use driver::Driver;

pub mod lex;
#[macro_use]
pub mod parse;
pub mod lowering;
pub mod analysis;
pub mod errors;
pub  mod  codemap;
mod driver;

fn main() {
    println!("Hello, world!");
}
