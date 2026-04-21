use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPhase {
    Lex,
    Parse,
    Runtime,
}

impl Display for ErrorPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Lex => "Lex",
            Self::Parse => "Parse",
            Self::Runtime => "Runtime",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticError {
    phase: ErrorPhase,
    line: usize,
    column: usize,
    message: String,
}

impl DiagnosticError {
    pub fn new(phase: ErrorPhase, line: usize, column: usize, message: impl Into<String>) -> Self {
        Self {
            phase,
            line,
            column,
            message: message.into(),
        }
    }

    pub fn phase(&self) -> ErrorPhase {
        self.phase
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for DiagnosticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} error at {}:{}: {}",
            self.phase, self.line, self.column, self.message
        )
    }
}

impl Error for DiagnosticError {}

pub type Result<T> = std::result::Result<T, DiagnosticError>;
