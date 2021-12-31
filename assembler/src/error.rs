use crate::token::Token;
use std::{fmt, io};

pub type Result<'s, T> = std::result::Result<T, Error<'s>>;

#[derive(Debug)]
pub enum Error<'s> {
    Scan {
        line: usize,
        lexeme: &'s str,
        message: &'s str,
    },
    Parse {
        token: &'s Token<'s>,
        message: &'s str,
    },
    Code {
        token: &'s Token<'s>,
        message: &'s str,
    },
    Io(io::Error),
}

impl<'s> Error<'s> {
    pub fn scan(line: usize, lexeme: &'s str, message: &'s str) -> Self {
        Self::Scan {
            line,
            lexeme,
            message,
        }
    }

    pub fn parse(token: &'s Token, message: &'s str) -> Self {
        Self::Parse { token, message }
    }

    pub fn code(token: &'s Token, message: &'s str) -> Self {
        Self::Code { token, message }
    }
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scan {
                line,
                lexeme,
                message,
            } => write!(f, "[line {}] scan error at `{}`: {}", line, lexeme, message),
            Self::Parse { token, message } => write!(
                f,
                "[line {}] parse error at `{}`: {}",
                token.line, token.lexeme, message
            ),
            Self::Code { token, message } => write!(
                f,
                "[line {}] code generation error at `{}`: {}",
                token.line, token.lexeme, message
            ),
            Self::Io(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for Error<'_> {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
