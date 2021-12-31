use crate::token::Token;
use std::str::FromStr;

#[derive(Debug)]
pub enum Instruction<'s> {
    A(&'s Token<'s>),
    C {
        comp: Computation,
        dest: Destination,
        jump: Jump,
    },
    Label(&'s Token<'s>),
}

#[derive(Clone, Copy, Debug)]
pub enum Computation {
    Zero,
    One,
    NegativeOne,
    D,
    A,
    M,
    NotD,
    NotA,
    NotM,
    NegativeD,
    NegativeA,
    NegativeM,
    DPlusOne,
    APlusOne,
    MPlusOne,
    DMinusOne,
    AMinusOne,
    MMinusOne,
    DPlusA,
    DPlusM,
    DMinusA,
    DMinusM,
    AMinusD,
    MMinusD,
    DAndA,
    DAndM,
    DOrA,
    DOrM,
}

#[derive(Clone, Copy, Debug)]
pub enum Destination {
    Null,
    M,
    D,
    Md,
    A,
    Am,
    Ad,
    Amd,
}

pub struct DestinationParseError;

impl FromStr for Destination {
    type Err = DestinationParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use Destination::*;
        let dest = match s {
            "M" => M,
            "D" => D,
            "MD" => Md,
            "A" => A,
            "AM" => Am,
            "AD" => Ad,
            "AMD" => Amd,
            _ => return Err(DestinationParseError),
        };
        Ok(dest)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Jump {
    Null,
    Unconditional,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
}

pub struct JumpParseError;

impl FromStr for Jump {
    type Err = JumpParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let jump = match s {
            "JMP" => Jump::Unconditional,
            "JGT" => Jump::Greater,
            "JGE" => Jump::GreaterEqual,
            "JLT" => Jump::Less,
            "JLE" => Jump::LessEqual,
            "JEQ" => Jump::Equal,
            "JNE" => Jump::NotEqual,
            _ => return Err(JumpParseError),
        };
        Ok(jump)
    }
}
