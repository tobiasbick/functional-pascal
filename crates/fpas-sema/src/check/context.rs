use crate::error::{SemaError, sema_error};
use crate::scope::ScopeStack;
use crate::types::Ty;
use fpas_diagnostics::DiagnosticCode;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::{HashMap, HashSet};

/// Maps expression identity (`Expr` as `*const Expr`) to its semantic type.
pub type ExprTypeMap = HashMap<usize, Ty>;

/// Maps a call-expression identity to its qualified method name (e.g. `Point.DistanceTo`).
/// Present only for calls that are record method invocations.
pub type MethodCallMap = HashMap<usize, String>;

pub struct Checker {
    pub(crate) scopes: ScopeStack,
    pub(crate) errors: Vec<SemaError>,
    pub(crate) expr_types: ExprTypeMap,
    pub(crate) method_calls: MethodCallMap,
    /// Canonical std unit names from `uses` (e.g. `Std.Console`).
    pub(crate) loaded_std_units: HashSet<String>,
    /// Short names that map to multiple fully-qualified std symbols (ambiguous).
    pub(crate) ambiguous_imports: HashMap<String, Vec<String>>,
    /// Unqualified `BuiltinStd` call -> fully qualified name for the polymorphic checker.
    pub(crate) short_builtin_redirect: HashMap<String, String>,
}

impl Checker {
    pub fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            errors: Vec::new(),
            expr_types: ExprTypeMap::new(),
            method_calls: MethodCallMap::new(),
            loaded_std_units: HashSet::new(),
            ambiguous_imports: HashMap::new(),
            short_builtin_redirect: HashMap::new(),
        }
    }

    pub fn finish(self) -> (Vec<SemaError>, ExprTypeMap, MethodCallMap) {
        (self.errors, self.expr_types, self.method_calls)
    }

    pub fn expr_lookup_key(expr: &Expr) -> usize {
        std::ptr::from_ref(expr) as usize
    }

    pub(crate) fn error_with_code(
        &mut self,
        code: DiagnosticCode,
        message: impl Into<String>,
        hint: impl Into<String>,
        span: Span,
    ) {
        self.errors.push(sema_error(code, message, hint, span));
    }
}
