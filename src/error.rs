use crate::{Token, TokenKind, lexer::Span};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseError {
    UnexpectedToken { expected: TokenKind, got: Token },
    UnexpectedIdent { expected: &'static str, span: Span },
    InvalidNumber(Span),
    Eof,
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken { expected, got } => write!(
                f,
                "unexpected token at {}, expecting '{expected:?}' got '{:?}'",
                got.span, got.kind
            ),
            Self::UnexpectedIdent { expected, span } => {
                write!(f, "unexpected identifier at {span}, expecting '{expected}'")
            }
            Self::InvalidNumber(span) => write!(f, "invalid number at {span}"),
            Self::Eof => write!(f, "unexpected EOF"),
        }
    }
}
