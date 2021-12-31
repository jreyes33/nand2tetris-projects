use crate::token::{Kind, Token};

pub struct Scanner<'s> {
    source: &'s str,
    current: usize,
    start: usize,
    line: usize,
    tokens: Vec<Token<'s>>,
}

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            current: 0,
            start: 0,
            line: 1,
            tokens: vec![],
        }
    }

    pub fn scan_tokens(&mut self) -> &[Token] {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        let eof = Token::new(Kind::Eof, self.line, "");
        self.tokens.push(eof);
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        use Kind::*;
        match c {
            '@' => self.add_token(At),
            '\n' => {
                self.add_token(LineBreak);
                self.line += 1;
            }
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '=' => self.add_token(Equal),
            ';' => self.add_token(Semicolon),
            '!' => self.add_token(Bang),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            '&' => self.add_token(Ampersand),
            '|' => self.add_token(Pipe),
            '/' => {
                if self.peek() == Some('/') {
                    while !self.is_at_end() && self.peek() != Some('\n') {
                        self.advance();
                    }
                }
            }
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' => self.identifier(),
            ' ' | '\r' | '\t' => (),
            // TODO: use Result error instead.
            _ => panic!("Unexpected character: {}", c),
        }
    }

    fn number(&mut self) {
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        // TODO: use Result error instead.
        let kind = Kind::Number(self.source[self.start..self.current].parse().unwrap());
        let token = Token::new(kind, self.line, &self.source[self.start..self.current]);
        self.tokens.push(token);
    }

    fn identifier(&mut self) {
        while matches!(
            self.peek(),
            Some('0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '.' | '$')
        ) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let kind = Kind::Identifier(text);
        let token = Token::new(kind, self.line, &self.source[self.start..self.current]);
        self.tokens.push(token);
    }

    fn add_token(&mut self, kind: Kind<'s>) {
        let token = Token::new(kind, self.line, &self.source[self.start..self.current]);
        self.tokens.push(token);
    }

    fn advance(&mut self) -> char {
        let mut char_indices = self.source[self.current..].char_indices().peekable();
        // TODO: use Result error instead.
        let (_, c) = char_indices.next().unwrap();
        let inc = if let Some((next_idx, _)) = char_indices.peek() {
            *next_idx
        } else {
            1
        };
        self.current += inc;
        c
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current..]
            .chars()
            .peekable()
            .peek()
            .copied()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
