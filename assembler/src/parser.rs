use crate::error::{Error, Result};
use crate::instruction::{Computation, Destination, Instruction, Jump};
use crate::token::Kind::*;
use crate::token::Token;

pub struct Parser<'s> {
    tokens: &'s [Token<'s>],
    current: usize,
}

impl<'s> Parser<'s> {
    pub fn new(tokens: &'s [Token<'s>]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Instruction>> {
        let mut instructions = vec![];
        loop {
            while matches!(self.peek().kind, LineBreak) {
                self.advance();
            }
            if self.is_at_end() {
                break;
            };
            instructions.push(self.instruction()?);
        }
        Ok(instructions)
    }

    fn instruction(&mut self) -> Result<'s, Instruction<'s>> {
        match self.peek().kind {
            LeftParen => self.label(),
            At => self.a_instruction(),
            _ => self.c_instruction(),
        }
    }

    fn label(&mut self) -> Result<'s, Instruction<'s>> {
        self.advance(); // The left paren
        let token = self.advance();
        let instruction = match token.kind {
            Identifier(_) => Instruction::Label(token),
            _ => return Err(Error::parse(token, "expect label name")),
        };
        let token = self.advance();
        match token.kind {
            RightParen => self.end_of_instruction()?,
            _ => return Err(Error::parse(token, "expect ')' after label name")),
        };
        Ok(instruction)
    }

    fn a_instruction(&mut self) -> Result<'s, Instruction<'s>> {
        self.advance(); // The @ sign
        let token = self.peek();
        let instruction = match token.kind {
            Number(_) | Identifier(_) => Instruction::A(self.advance()),
            _ => return Err(Error::parse(token, "expect number or identifier after '@'")),
        };
        self.end_of_instruction()?;
        Ok(instruction)
    }

    fn c_instruction(&mut self) -> Result<'s, Instruction<'s>> {
        let dest = self.destination()?;
        let comp = self.computation()?;
        let jump = self.jump()?;
        let instruction = Instruction::C { dest, comp, jump };
        self.end_of_instruction()?;
        Ok(instruction)
    }

    fn destination(&mut self) -> Result<'s, Destination> {
        if let Some(next_token) = self.peek_next() {
            match (&self.peek().kind, &next_token.kind) {
                (Identifier(ident), Equal) => {
                    let token = self.advance(); // Identifier
                    self.advance(); // Equals sign
                    ident
                        .parse()
                        .map_err(|_| Error::parse(token, "unknown destination"))
                }
                _ => Ok(Destination::Null),
            }
        } else {
            Ok(Destination::Null)
        }
    }

    fn computation(&mut self) -> Result<'s, Computation> {
        use Computation::*;
        let token = self.advance();
        let comp = match token.kind {
            Number(0) => Zero,
            Number(1) => One,
            Number(_) => return Err(Error::parse(token, "expect 0 or 1")),
            Minus => {
                let token = self.advance();
                match token.kind {
                    Number(1) => NegativeOne,
                    Identifier("A") => NegativeA,
                    Identifier("D") => NegativeD,
                    Identifier("M") => NegativeM,
                    _ => return Err(Error::parse(token, "expect 1, A, D, or M")),
                }
            }
            Bang => {
                let token = self.advance();
                match token.kind {
                    Identifier("A") => NotA,
                    Identifier("D") => NotD,
                    Identifier("M") => NotM,
                    _ => return Err(Error::parse(token, "expect A, D, or M")),
                }
            }
            Identifier(ident @ ("A" | "D" | "M")) => {
                let token = self.peek();
                if matches!(token.kind, LineBreak | Eof | Semicolon) {
                    match ident {
                        "A" => A,
                        "D" => D,
                        "M" => M,
                        _ => unreachable!("ident can only be A, D, or M"),
                    }
                } else if let Some(next_token) = self.peek_next() {
                    self.advance(); // Second token of computation
                    self.advance(); // Third token of computation
                    match (ident, &token.kind, &next_token.kind) {
                        ("A", Plus, Number(1)) => APlusOne,
                        ("A", Minus, Number(1)) => AMinusOne,
                        ("A", Minus, Identifier("D")) => AMinusD,
                        ("M", Plus, Number(1)) => MPlusOne,
                        ("M", Minus, Number(1)) => MMinusOne,
                        ("M", Minus, Identifier("D")) => MMinusD,
                        ("D", Plus, Number(1)) => DPlusOne,
                        ("D", Plus, Identifier("A")) => DPlusA,
                        ("D", Plus, Identifier("M")) => DPlusM,
                        ("D", Minus, Number(1)) => DMinusOne,
                        ("D", Minus, Identifier("A")) => DMinusA,
                        ("D", Minus, Identifier("M")) => DMinusM,
                        ("D", Ampersand, Identifier("A")) => DAndA,
                        ("D", Ampersand, Identifier("M")) => DAndM,
                        ("D", Pipe, Identifier("A")) => DOrA,
                        ("D", Pipe, Identifier("M")) => DOrM,
                        _ => return Err(Error::parse(next_token, "unknown computation")),
                    }
                } else {
                    return Err(Error::parse(token, "unknown computation"));
                }
            }
            Identifier(_) => return Err(Error::parse(token, "expect A, D, or M")),
            _ => return Err(Error::parse(token, "unknown computation")),
        };

        // Ensure end of computation.
        let token = self.peek();
        match token.kind {
            LineBreak | Eof | Semicolon => Ok(comp),
            _ => Err(Error::parse(token, "expect ';' or end of instruction")),
        }
    }

    fn jump(&mut self) -> Result<'s, Jump> {
        let token = self.peek();
        match token.kind {
            Semicolon => {
                self.advance(); // The semicolon
                let jump_token = self.advance();
                match jump_token.kind {
                    Identifier(ident) => ident
                        .parse()
                        .map_err(|_| Error::parse(jump_token, "unknown jump")),
                    _ => Err(Error::parse(jump_token, "expect jump")),
                }
            }
            _ => Ok(Jump::Null),
        }
    }

    fn end_of_instruction(&mut self) -> Result<'s, ()> {
        let token = self.advance();
        match token.kind {
            LineBreak | Eof => Ok(()),
            _ => Err(Error::parse(token, "expect end of instruction")),
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, Eof)
    }

    fn advance(&mut self) -> &'s Token<'s> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &'s Token<'s> {
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> &'s Token<'s> {
        &self.tokens[self.current]
    }

    fn peek_next(&self) -> Option<&'s Token<'s>> {
        self.tokens.get(self.current + 1)
    }
}
