//! Runtime error type.

use boa_cat::Error as EngineError;

/// All errors the runtime can produce.  Currently a thin wrapper around
/// [`boa_cat::Error`] since the runtime itself only contributes native
/// callables whose failures surface as `Outcome::Throw` values rather
/// than `Error`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// An error from the boa-cat engine (lex/parse/syntax/fuel/uncaught
    /// exception).
    Engine(EngineError),
}

impl From<EngineError> for Error {
    fn from(value: EngineError) -> Self {
        Self::Engine(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Engine(e) => write!(f, "engine error: {e}"),
        }
    }
}

impl std::error::Error for Error {}
