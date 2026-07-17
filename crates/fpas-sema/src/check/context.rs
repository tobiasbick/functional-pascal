use crate::error::{SemaError, sema_error};
use crate::scope::ScopeStack;
use crate::types::Ty;
use fpas_diagnostics::DiagnosticCode;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::{HashMap, HashSet};

/// Maps expression identity (`Expr` as `*const Expr`) to its semantic type.
pub type ExprTypeMap = HashMap<usize, Ty>;

/// How a record member call should be lowered by the compiler.
///
/// **Documentation:** `docs/pascal/language/types/record-methods.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodCallTarget {
    /// Instance method: emit the receiver, then the explicit arguments.
    Instance(String),
    /// Static function: emit only the explicit arguments (no receiver).
    Static(String),
}

impl MethodCallTarget {
    /// Qualified callable name used for `Op::Call` (e.g. `Point.Create`).
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        match self {
            Self::Instance(name) | Self::Static(name) => name,
        }
    }
}

/// Maps a call-expression (or call-statement designator) identity to its
/// resolved record member call target.
///
/// Ordinary [`Expr::Call`](fpas_parser::Expr::Call) entries use [`crate::expr_lookup_key`]
/// (or [`crate::designator_lookup_key`] for call statements). Postfix
/// [`PostfixOperation::MethodCall`](fpas_parser::PostfixOperation::MethodCall) entries use
/// [`crate::postfix_operation_lookup_key`] instead.
pub type MethodCallMap = HashMap<usize, MethodCallTarget>;

/// Maps a named record type to its ordered field list, each entry carrying an optional
/// cloned default expression. The order matches the type definition.
///
/// **Documentation:** `docs/pascal/language/types/records.md` (Default field values)
pub type RecordDefaultsMap = HashMap<String, Vec<(String, Option<Expr>)>>;

/// Marks `CaseLabel::Value.start` expressions that semantic analysis interpreted
/// as scalar guard bindings instead of value labels.
pub type ScalarCaseBindingMap = HashSet<usize>;

pub struct Checker {
    pub(crate) scopes: ScopeStack,
    pub(crate) errors: Vec<SemaError>,
    pub(crate) expr_types: ExprTypeMap,
    pub(crate) method_calls: MethodCallMap,
    /// Canonical std unit names from `uses` (e.g. `Std.Console`).
    pub(crate) loaded_std_units: HashSet<String>,
    /// Short names that map to multiple fully-qualified std symbols (ambiguous).
    pub(crate) ambiguous_imports: HashMap<String, Vec<String>>,
    /// Unqualified enum variant names that map to multiple `Type.Variant` symbols (ambiguous).
    pub(crate) ambiguous_enum_variants: HashMap<String, Vec<String>>,
    /// Canonical short enum variant names registered at the program root without ambiguity.
    pub(crate) enum_short_variant_keys: HashMap<String, String>,
    /// Unqualified `BuiltinStd` call -> fully qualified name for the polymorphic checker.
    pub(crate) short_builtin_redirect: HashMap<String, String>,
    /// Canonical short names inserted at the program root by [`crate::std_registry::register_short_aliases`].
    pub(crate) std_short_alias_keys: HashSet<String>,
    /// Named record type → ordered (field_name, optional_default_expr) pairs.
    pub(crate) record_defaults: RecordDefaultsMap,
    /// `case` label expressions that bind the scrutinee for a guarded scalar arm.
    pub(crate) scalar_case_bindings: ScalarCaseBindingMap,
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
            ambiguous_enum_variants: HashMap::new(),
            enum_short_variant_keys: HashMap::new(),
            short_builtin_redirect: HashMap::new(),
            std_short_alias_keys: HashSet::new(),
            record_defaults: RecordDefaultsMap::new(),
            scalar_case_bindings: ScalarCaseBindingMap::new(),
        }
    }

    pub fn finish(
        self,
    ) -> (
        Vec<SemaError>,
        ExprTypeMap,
        MethodCallMap,
        RecordDefaultsMap,
        ScalarCaseBindingMap,
    ) {
        (
            self.errors,
            self.expr_types,
            self.method_calls,
            self.record_defaults,
            self.scalar_case_bindings,
        )
    }

    /// Stable identity key for an AST expression node.
    ///
    /// Uses the memory address of the `Expr` reference. This is sound because:
    /// - The AST (`Program`) is immutable and heap-allocated for the entire analysis.
    /// - No AST nodes are moved or cloned during checking.
    /// - Keys are only used within a single `check_program` call.
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

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}
