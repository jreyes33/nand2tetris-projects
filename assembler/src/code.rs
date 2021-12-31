use crate::error::{Error, Result};
use crate::instruction::{Computation, Destination, Instruction, Jump};
use crate::token::Kind;
use std::collections::HashMap;

pub struct Generator<'s> {
    current: usize,
    next_variable: u16,
    instructions: &'s [Instruction<'s>],
    symbols: HashMap<&'s str, u16>,
}

impl<'s> Generator<'s> {
    pub fn new(instructions: &'s [Instruction<'s>]) -> Self {
        let symbols = HashMap::from([
            ("R0", 0),
            ("R1", 1),
            ("R2", 2),
            ("R3", 3),
            ("R4", 4),
            ("R5", 5),
            ("R6", 6),
            ("R7", 7),
            ("R8", 8),
            ("R9", 9),
            ("R10", 10),
            ("R11", 11),
            ("R12", 12),
            ("R13", 13),
            ("R14", 14),
            ("R15", 15),
            ("SCREEN", 16384),
            ("KBD", 24576),
            ("SP", 0),
            ("LCL", 1),
            ("ARG", 2),
            ("THIS", 3),
            ("THAT", 4),
        ]);
        Self {
            instructions,
            symbols,
            current: 0,
            next_variable: 16,
        }
    }

    pub fn register_labels(&mut self) -> Result<()> {
        let mut pending = vec![];
        let mut i = 0;
        for instruction in self.instructions {
            match instruction {
                Instruction::Label(token) => match token.kind {
                    Kind::Identifier(label) => pending.push(label),
                    _ => return Err(Error::code(token, "expect label identifier")),
                },
                _ => {
                    while let Some(label) = pending.pop() {
                        self.symbols.insert(label, i);
                    }
                    i += 1;
                }
            }
        }
        Ok(())
    }

    fn translate(&mut self, instruction: &'s Instruction) -> Result<u16> {
        match instruction {
            Instruction::A(token) => match token.kind {
                Kind::Number(n) => Ok(translate_a_number(n)),
                Kind::Identifier(label) => Ok(self.translate_a_label(label)),
                _ => Err(Error::code(token, "expect number or identifier")),
            },
            Instruction::C { dest, comp, jump } => Ok(translate_c(*dest, *comp, *jump)),
            Instruction::Label(token) => Err(Error::code(token, "can't translate a label")),
        }
    }

    fn translate_a_label(&mut self, label: &'s str) -> u16 {
        let n = match self.symbols.get(label) {
            Some(n) => *n,
            None => {
                let n = self.next_variable;
                self.next_variable += 1;
                self.symbols.insert(label, n);
                n
            }
        };
        translate_a_number(n)
    }
}

fn translate_a_number(n: u16) -> u16 {
    n & 0b0111111111111111
}

fn translate_c(dest: Destination, comp: Computation, jump: Jump) -> u16 {
    let n = translate_jump(jump) | translate_comp(comp) | translate_dest(dest);
    n | 0b1110000000000000
}

fn translate_jump(jump: Jump) -> u16 {
    use Jump::*;
    match jump {
        Null => 0b000,
        Greater => 0b001,
        Equal => 0b010,
        GreaterEqual => 0b011,
        Less => 0b100,
        NotEqual => 0b101,
        LessEqual => 0b110,
        Unconditional => 0b111,
    }
}

fn translate_comp(comp: Computation) -> u16 {
    use Computation::*;
    let n = match comp {
        Zero => 0b0101010,
        One => 0b0111111,
        NegativeOne => 0b0111010,
        D => 0b0001100,
        A => 0b0110000,
        M => 0b1110000,
        NotD => 0b0001101,
        NotA => 0b0110001,
        NotM => 0b1110001,
        NegativeD => 0b0001111,
        NegativeA => 0b0110011,
        NegativeM => 0b1110011,
        DPlusOne => 0b0011111,
        APlusOne => 0b0110111,
        MPlusOne => 0b1110111,
        DMinusOne => 0b0001110,
        AMinusOne => 0b0110010,
        MMinusOne => 0b1110010,
        DPlusA => 0b0000010,
        DPlusM => 0b1000010,
        DMinusA => 0b0010011,
        DMinusM => 0b1010011,
        AMinusD => 0b0000111,
        MMinusD => 0b1000111,
        DAndA => 0b0000000,
        DAndM => 0b1000000,
        DOrA => 0b0010101,
        DOrM => 0b1010101,
    };
    n << 6
}

fn translate_dest(dest: Destination) -> u16 {
    use Destination::*;
    let n = match dest {
        Null => 0b000,
        M => 0b001,
        D => 0b010,
        Md => 0b011,
        A => 0b100,
        Am => 0b101,
        Ad => 0b110,
        Amd => 0b111,
    };
    n << 3
}

impl Iterator for Generator<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(instruction) = self.instructions.get(self.current) {
                self.current += 1;
                match instruction {
                    Instruction::Label(_) => continue,
                    _ => {
                        break Some(
                            self.translate(instruction)
                                .expect("translation shouldn't fail"),
                        )
                    }
                }
            } else {
                break None;
            }
        }
    }
}
