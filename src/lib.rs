#![allow(unused)] // REMOVE WHEN DONE

mod lexer;
mod parser;
mod ast;
mod generators;
pub mod error;

mod __exports {
    use crate::lexer;
    pub use lexer::{Token, Span, TokenKind};
    use crate::generators;

    pub use generators::gen_c::*;
}

pub use __exports::*;