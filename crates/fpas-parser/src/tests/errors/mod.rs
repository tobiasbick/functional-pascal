use super::{parse_compilation_unit_with_errors, parse_with_errors};
use crate::ParseDiagnostic;

mod api;
mod chained_comparison;
mod delimiters;
mod diagnostics;
mod keywords;
mod nesting;
mod parameters;
mod recovery;
mod reserved_keywords;
mod statement_separators;
mod syntax;
mod synthetic_eof;
mod trailing_input;
mod uses;
