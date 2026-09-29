use std::fmt;

#[derive(Debug)]
pub enum DataError {
    Message(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for DataError {}

pub type Result<T> = std::result::Result<T, DataError>;
