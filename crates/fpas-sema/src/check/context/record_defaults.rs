//! Checked default expressions and imported initializer identities.

use fpas_parser::Expr;
use std::{collections::HashMap, sync::Arc};

/// Default evaluation retained without copying AST expression identities.
#[derive(Debug, Clone)]
pub enum RecordDefault {
    /// Original checked expression, or an imported canonical scalar expression.
    Expression(Arc<Expr>),
    /// Internal callable in the declaring compiled unit.
    Initializer(String),
}

impl RecordDefault {
    /// Access a source expression when static evaluation is possible.
    pub fn expression(&self) -> Option<&Expr> {
        match self {
            Self::Expression(expression) => Some(expression),
            Self::Initializer(_) => None,
        }
    }
}

/// Defaults in field declaration order, indexed by nominal record identity.
pub type RecordDefaultsMap = HashMap<String, Vec<(String, Option<RecordDefault>)>>;
