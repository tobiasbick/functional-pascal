//! Structured filesystem failures retained until the FPAS boundary.
use std::io;

#[derive(Debug)]
pub(super) enum Failure {
    Io(io::Error),
    AtPath(String, io::Error),
    EncodingAtPath(String, String),
    InvalidArgument(&'static str, String),
    InvalidPattern(String),
    InvalidEncoding(String),
    LimitExceeded(&'static str, i64),
}
impl Failure {
    pub(super) fn at(path: &std::path::Path, error: io::Error) -> Self {
        Self::AtPath(path.to_string_lossy().into_owned(), error)
    }
}
impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) | Self::AtPath(_, e) => e.fmt(f),
            Self::EncodingAtPath(_, detail) => detail.fmt(f),
            Self::InvalidArgument(_, reason)
            | Self::InvalidPattern(reason)
            | Self::InvalidEncoding(reason) => reason.fmt(f),
            Self::LimitExceeded(operation, limit) => {
                write!(f, "{operation} exceeds maximum limit of {limit}")
            }
        }
    }
}
