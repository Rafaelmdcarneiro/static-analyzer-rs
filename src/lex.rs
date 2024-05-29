//! Module for performing lexical analysis on source code.

use std::str;

use crate::codemap::Span;
use crate::errors::*;

struct Tokenizer<'a> {
    current_index: usize,
    remaining_text: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &str) -> Tokenizer {
        Tokenizer {
            current_index: 0,
            remaining_text: src,
        }
    }

    fn next_token(&mut self) -> Result<Option<(TokenKind, usize, usize)>> {
        self.skip_whitespace();

        if self.remaining_text.is_empty() {
            Ok(None)
        } else {
            let start = self.current_index;
            let tok = self._next_token()
                .chain_err(|| ErrorKind::MessageWithLocation(self.current_index,
                                                             "Couldn't read the next token"))?;
            let end = self.current_index;
            Ok(Some((tok, start, end)))
        }
    }

    fn skip_whitespace(&mut self) {
        let skipped = skip(self.remaining_text);
        self.chomp(skipped);
    }

    fn _next_token(&mut self) -> Result<TokenKind> {
        let (tok, bytes_read) = tokenize_single_token(self.remaining_text)?;
        self.chomp(bytes_read);

        Ok(tok)
    }

    fn chomp(&mut self, num_bytes: usize) {
        self.remaining_text = &self.remaining_text[num_bytes..];
        self.current_index += num_bytes;
    }
}

/// Turn a string of valid Delphi code into a list of tokens, including the
/// location of that token's start and end point in the original source code.
///
/// Note the token indices represent the half-open interval `[start, end)`,
/// equivalent to `start .. end` in Rust.
pub fn tokenize(src: &str) -> Result<Vec<(TokenKind, usize, usize)>> {
    let mut tokenizer = Tokenizer::new(src);
    let mut tokens = Vec::new();

    while let Some(tok) = tokenizer.next_token()? {
        tokens.push(tok);
    }

    Ok(tokens)
}

/// Any valid token in the Delphi programming language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(tag = "type")]
pub enum TokenKind {
    Integer(usize),
    Decimal(f64),
    Identifier(String),
    QuotedString(String),
    Asterisk,
    At,
    Carat,
    CloseParen,
    CloseSquare,
    Colon,
    Dot,
    End,
    Equals,
    Minus,
    OpenParen,
    OpenSquare,
    Plus,
    Semicolon,
    Slash,
}

impl From<String> for TokenKind {
    fn from(other: String) -> TokenKind {
        TokenKind::Identifier(other)
    }
}

impl<'a> From<&'a str> for TokenKind {
    fn from(other: &'a str) -> TokenKind {
        TokenKind::Identifier(other.to_string())
    }
}

impl From<usize> for TokenKind {
    fn from(other: usize) -> TokenKind {
        TokenKind::Integer(other)
    }
}

impl From<f64> for TokenKind {
    fn from(other: f64) -> TokenKind {
        TokenKind::Decimal(other)
    }
}

/// Try to lex a single token from the input stream.
pub fn tokenize_single_token(data: &str) -> Result<(TokenKind, usize)> {
    let next = match data.chars().next() {
        Some(c) => c,
        None => bail!(ErrorKind::UnexpectedEOF),
    };

    let (tok, length) = match next {
        '.' => (TokenKind::Dot, 1),
        '=' => (TokenKind::Equals, 1),
        '+' => (TokenKind::Plus, 1),
        '-' => (TokenKind::Minus, 1),
        '*' => (TokenKind::Asterisk, 1),
        '/' => (TokenKind::Slash, 1),
        '@' => (TokenKind::At, 1),
        '^' => (TokenKind::Carat, 1),
        '(' => (TokenKind::OpenParen, 1),
        ')' => (TokenKind::CloseParen, 1),
        '[' => (TokenKind::OpenSquare, 1),
        ']' => (TokenKind::CloseSquare, 1),
        '0'..='9' => tokenize_number(data).chain_err(|| "Couldn't tokenize a number")?,
        c @ '_' | c if c.is_alphabetic() => tokenize_ident(data)
            .chain_err(|| "Couldn't tokenize an identifier")?,
        other => bail!(ErrorKind::UnknownCharacter(other)),
    };

    Ok((tok, length))
}

/// Consumes bytes while a predicate evaluates to true.
fn take_while<F>(data: &str, mut pred: F) -> Result<(&str, usize)>
    where F: FnMut(char) -> bool
{
    let mut current_index = 0;

    for ch in data.chars() {
        let should_continue = pred(ch);

        if !should_continue {
            break;
        }

        current_index += ch.len_utf8();
    }

    if current_index == 0 {
        Err("No Matches".into())
    } else {
        Ok((&data[..current_index], current_index))
    }
}

fn tokenize_ident(data: &str) -> Result<(TokenKind, usize)> {
    // identifiers can't start with a number
    match data.chars().next() {
        Some(ch) if ch.is_digit(10) => bail!("Identifiers can't start with a number"),
        None => bail!(ErrorKind::UnexpectedEOF),
        _ => {}
    }

    let (got, bytes_read) = take_while(data, |ch| ch == '_' || ch.is_alphanumeric())?;

    // TODO: Recognise keywords using a `match` statement here.

    let tok = TokenKind::Identifier(got.to_string());
    Ok((tok, bytes_read))
}

/// Tokenize a numeric literal.
fn tokenize_number(data: &str) -> Result<(TokenKind, usize)> {
    let mut seen_dot = false;

    let (decimal, bytes_read) = take_while(data, |c| {
        if c.is_digit(10) {
            true
        } else if c == '.' {
            if !seen_dot {
                seen_dot = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    })?;

    if seen_dot {
        let n: f64 = decimal.parse()?;
        Ok((TokenKind::Decimal(n), bytes_read))
    } else {
        let n: usize = decimal.parse()?;
        Ok((TokenKind::Integer(n), bytes_read))
    }
}

/// Skip past any whitespace characters or comments.
fn skip(src: &str) -> usize {
    let mut remaining = src;

    loop {
        let ws = skip_whitespace(remaining);
        remaining = &remaining[ws..];
        let comments = skip_comments(remaining);
        remaining = &remaining[comments..];

        if ws + comments == 0 {
            return src.len() - remaining.len();
        }
    }
}

fn skip_whitespace(data: &str) -> usize {
    match take_while(data, |ch| ch.is_whitespace()) {
        Ok((_, bytes_skipped)) => bytes_skipped,
        _ => 0,
    }
}

fn skip_comments(src: &str) -> usize {
    let pairs = [("//", "\n"), ("{", "}"), ("(*", "*)")];

    for &(pattern, matcher) in &pairs {
        if src.starts_with(pattern) {
            let leftovers = skip_until(src, matcher);
            return src.len() - leftovers.len();
        }
    }

    0
}

fn skip_until<'a>(mut src: &'a str, pattern: &str) -> &'a str {
    while !src.is_empty() && !src.starts_with(pattern) {
        let next_char_size = src.chars().next().expect("The string isn't empty").len_utf8();
        src = &src[next_char_size..];
    }

    &src[pattern.len()..]
}

/// A valid Delphi source code token.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// The token's location relative to the rest of the files being
    /// processed.
    pub span: Span,
    /// What kind of token is this?
    pub kind: TokenKind,
}

impl Token {
    /// Create a new token out of a `Span` and something which can be turned
    /// into a `TokenKind`.
    pub fn new<K: Into<TokenKind>>(span: Span, kind: K) -> Token {
        let kind = kind.into();
        Token { span, kind }
    }
}

impl<T> From<T> for Token
    where T: Into<TokenKind> {
    fn from(other: T) -> Token {
        Token::new(Span::dummy(), other)
    }
}