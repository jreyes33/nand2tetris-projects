#[derive(Debug)]
pub struct Token<'s> {
    pub kind: Kind<'s>,
    pub line: usize,
    pub lexeme: &'s str,
}

impl<'s> Token<'s> {
    pub fn new(kind: Kind<'s>, line: usize, lexeme: &'s str) -> Self {
        Self { kind, line, lexeme }
    }
}

#[rustfmt::skip]
#[derive(Debug)]
pub enum Kind<'s> {
    At,
    Number(u16), Identifier(&'s str),
    LineBreak,
    LeftParen, RightParen,
    Equal, Semicolon,
    Bang, Minus, Plus, Ampersand, Pipe,
    Eof,
}
