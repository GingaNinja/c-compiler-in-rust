use crate::lex::{Keyword, Token};

pub enum DccError {
    PreProcess,
    ExeCreate,
    ExtraTokens,
    ExpectedToken { actual: Token, expected: String },
    ExpectedKeyword { actual: Token, expected: Keyword },
    ExpectedMoreTokens { expected: String },
    ExpectedMoreKeywords { expected: Keyword },
    InvalidInputChar { pos: usize, found_char: String },
    RegexError,
}

impl From<regex::Error> for DccError {
    fn from(_value: regex::Error) -> Self {
        Self::RegexError
    }
}

impl std::fmt::Debug for DccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Display for DccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreProcess => write!(f, "issue running pre-process"),
            Self::ExeCreate => write!(f, "issue creating the exe"),
            Self::ExtraTokens => write!(f, "extra tokens at the end of the file"),
            Self::ExpectedToken { actual, expected } => {
                write!(f, "Expected '{expected}' but found '{actual}'")
            }
            Self::ExpectedMoreTokens { expected } => {
                write!(f, "Expected '{expected}' but reached the end")
            }
            Self::InvalidInputChar { pos, found_char } => {
                write!(f, "invalid input, char {pos} - {found_char}")
            }
            Self::ExpectedKeyword { actual, expected } => {
                write!(f, "Expected '{expected}' but found '{actual}'")
            }
            Self::ExpectedMoreKeywords { expected } => {
                write!(f, "Expected '{expected}' but reached the end")
            }
            Self::RegexError => {
                write!(f, "Regex error - bad...")
            }
        }
    }
}
