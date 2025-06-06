use crate::{
    ast::*,
    error::ParseError,
    lexer::{Lexer, Span, Token, TokenKind},
};

pub fn parse(source: &str) -> Result<File, ParseError> {
    Parser::new(source).parse_file()
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Token>,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let lookahead = lexer.next();
        Self {
            lexer,
            lookahead,
            source,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.lookahead.as_ref()
    }

    fn next(&mut self) -> Option<Token> {
        let next = self.lookahead.take();
        self.lookahead = self.advance();
        next
    }

    fn advance(&mut self) -> Option<Token> {
        if let Some(tok) = self.lexer.next() {
            return Some(tok);
        }
        None
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        match self.peek() {
            Some(tok) if tok.kind == kind => Ok(self.next().unwrap()),
            Some(tok) => Err(ParseError::UnexpectedToken {
                expected: kind,
                got: *tok,
            }),
            None => Err(ParseError::Eof),
        }
    }

    // For debugging or error recovery
    fn slice(&self, span: &Span) -> &str {
        &self.source[span.start.offset..span.end.offset]
    }
}

impl<'a> Parser<'a> {
    pub fn parse_file(&mut self) -> Result<File, ParseError> {
        let mut defs = Vec::new();

        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Ident => {
                    let kw = self.slice(&tok.span);
                    if kw == "message" {
                        defs.push(TopLevel::Message(self.parse_message().unwrap()));
                    } else if kw == "bitfield" {
                        defs.push(TopLevel::Bitfield(self.parse_bitfield().unwrap()));
                    } else {
                        return Err(ParseError::UnexpectedIdent {
                            expected: "message",
                            span: tok.span,
                        });
                    }
                }
                TokenKind::Eof => break,
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: TokenKind::Ident,
                        got: *tok,
                    });
                }
            }
        }

        Ok(File { defs })
    }

    fn parse_message(&mut self) -> Result<Message, ParseError> {
        let msg_kw = self.expect(TokenKind::Ident)?;
        if self.slice(&msg_kw.span) != "message" {
            return Err(ParseError::UnexpectedToken {
                expected: TokenKind::Ident,
                got: msg_kw,
            });
        }

        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        while self.peek().is_some_and(|t| t.kind != TokenKind::RBrace) {
            fields.push(self.parse_field()?);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(Message { name, fields })
    }

    fn parse_bitfield(&mut self) -> Result<Bitfield, ParseError> {
        let kw = self.expect(TokenKind::Ident)?;
        if self.slice(&kw.span) != "bitfield" {
            return Err(ParseError::UnexpectedToken {
                expected: TokenKind::Ident,
                got: kw,
            });
        }

        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut flags = Vec::new();
        while self.peek().is_some_and(|t| t.kind != TokenKind::RBrace) {
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let num = self.expect(TokenKind::Number)?;
            let offset = match self.slice(&num.span).parse::<u8>() {
                Ok(offset) => offset,
                Err(_) => return Err(ParseError::InvalidNumber(num.span)),
            };
            self.expect(TokenKind::Comma)?;
            flags.push(BitFlag { name, offset });
        }

        self.expect(TokenKind::RBrace)?;
        Ok(Bitfield { name, flags })
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let decorator = if self.peek().is_some_and(|t| t.kind == TokenKind::At) {
            self.next();
            Some(self.expect_ident()?)
        } else {
            None
        };

        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        self.expect(TokenKind::Comma)?;

        Ok(Field {
            decorator,
            name,
            ty,
        })
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        let tok = self.next().ok_or(ParseError::Eof)?;
        let base = match tok.kind {
            TokenKind::Ident => match self.slice(&tok.span) {
                "u8" => TypeIdent::Native(NativeType::U8),
                "u16" => TypeIdent::Native(NativeType::U16),
                "u32" => TypeIdent::Native(NativeType::U32),
                "u64" => TypeIdent::Native(NativeType::U64),
                "i8" => TypeIdent::Native(NativeType::I8),
                "i16" => TypeIdent::Native(NativeType::I16),
                "i32" => TypeIdent::Native(NativeType::I32),
                "i64" => TypeIdent::Native(NativeType::I64),
                "vu32" => TypeIdent::Native(NativeType::VU32),
                "vu64" => TypeIdent::Native(NativeType::VU64),
                "vi32" => TypeIdent::Native(NativeType::VI32),
                "vi64" => TypeIdent::Native(NativeType::VI64),
                name => TypeIdent::Custom(name.to_string()),
            },
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: TokenKind::Ident,
                    got: tok,
                });
            }
        };

        if self.peek().is_some_and(|t| t.kind == TokenKind::LBracket) {
            self.next(); // consume [
            match self.peek().map(|t| t.kind) {
                Some(TokenKind::RBracket) => {
                    self.next();
                    Ok(TypeExpr::ArrayNoField(base))
                }
                Some(TokenKind::Ident) => {
                    let size_tok = self.next().unwrap();
                    let size = self.slice(&size_tok.span).to_string();
                    self.expect(TokenKind::RBracket)?;
                    Ok(TypeExpr::ArrayWithField(base, size))
                }
                _ => Err(ParseError::UnexpectedToken {
                    expected: TokenKind::Ident,
                    got: self.peek().cloned().unwrap_or(Token {
                        kind: TokenKind::Eof,
                        span: Span::default(),
                    }),
                }),
            }
        } else {
            Ok(TypeExpr::Ident(base))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let tok = self.expect(TokenKind::Ident)?;
        Ok(self.slice(&tok.span).to_string())
    }
}
