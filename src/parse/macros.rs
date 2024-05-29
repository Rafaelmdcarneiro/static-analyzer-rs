/// Shorthand macro for generating a token from *anything* which can be
/// converted into a `TokenKind`, or any of the `TokenKind` variants.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate static_analyser;
///
/// # fn main() {
/// tok!(Dot);
/// tok!(123);
/// tok!(3.14);
/// tok!(OpenParen);
/// # }
/// ```
#[macro_export]
macro_rules! tok {
    ($thing:tt) =>  {
        {
            #[allow(unused_imports)]
            use $crate::lex::TokenKind::*;
            $crate::lex::Token::from($thing)
        }
    };
}