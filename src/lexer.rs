use std::str::Chars;

// Define the tokens you want to recognize
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Ident,
    Number,
    Colon,
    Comma,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    At,
    Eof,
    Unknown,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub(crate) struct Lexer<'a> {
    chars: Chars<'a>,
    start: LexerPos,
    curr: LexerPos,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars(),
            start: LexerPos::default(),
            curr: LexerPos::default(),
        }
    }

    fn next_char(&mut self) -> char {
        match self.chars.next() {
            None => '\0',
            Some(c) => {
                self.curr.increase_by_char_len(c);
                c
            }
        }
    }

    #[inline(always)]
    fn peek_char(&self, n: usize) -> char {
        self.chars.clone().nth(n).unwrap_or('\0')
    }

    fn token(&mut self, kind: TokenKind) -> Token {
        let token = Token {
            kind,
            span: Span {
                start: self.start.into(),
                end: self.curr.into(),
            },
        };
        // after a new "token" is pushed, reset "start" position to "curr" position
        self.start = self.curr;

        token
    }

    fn skip_ws(&mut self) {
        while self.peek_char(0).is_whitespace() {
            self.next_char();
        }
        self.start = self.curr;
    }

    fn next_token(&mut self) -> Token {
        self.skip_ws();
        match self.next_char() {
            '\0' => self.token(TokenKind::Eof),
            ':' => self.token(TokenKind::Colon),
            ',' => self.token(TokenKind::Comma),
            '{' => self.token(TokenKind::LBrace),
            '}' => self.token(TokenKind::RBrace),
            '[' => self.token(TokenKind::LBracket),
            ']' => self.token(TokenKind::RBracket),
            '@' => self.token(TokenKind::At),
            '/' if self.peek_char(0) == '/' => {
                self.next_char(); // consume second '/'
                while self.peek_char(0) != '\n' {
                    self.next_char();
                }
                self.start = self.curr;
                self.next_token()
            }
            c if is_id_start(c) => {
                self.advance_while(is_id_continue);
                self.token(TokenKind::Ident)
            }
            c if c.is_ascii_digit() => self.token(TokenKind::Number),
            _ => self.token(TokenKind::Unknown),
        }
    }

    fn advance_while(&mut self, predicate: fn(char) -> bool) {
        loop {
            let c = self.peek_char(0);
            if c == '\0' || !predicate(c) {
                break;
            }
            self.next_char();
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next_token();
        if next.kind == TokenKind::Eof {
            return None;
        }
        Some(next)
    }
}

#[inline]
fn is_id_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

#[inline]
fn is_id_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.start.line + 1, self.start.column + 1)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Pos {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

#[derive(Default, Clone, Copy)]
struct LexerPos {
    line: u32,
    col: u32,
    off: usize,
}

impl LexerPos {
    fn increase_by_char_len(&mut self, c: char) {
        let len = c.len_utf8();
        self.col += 1;
        self.off += len;
        if c == '\n' {
            self.line += 1;
            self.col = 0;
        }
    }
}

impl From<LexerPos> for Pos {
    fn from(value: LexerPos) -> Self {
        Self {
            line: value.line,
            column: value.col,
            offset: value.off,
        }
    }
}
