use super::closures::{ClosureInfoMap, NestedRoutineCaptureMap};
use crate::error::{SemaError, sema_error};
use crate::scope::ScopeStack;
use crate::types::Ty;
use fpas_diagnostics::DiagnosticCode;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};

/// Maps expression identity (`Expr` as `*const Expr`) to its semantic type.
pub type ExprTypeMap = HashMap<usize, Ty>;

/// Maps a call expression or call-statement designator to its canonical `Std.*` dispatch name.
pub type IntrinsicCallMap = HashMap<usize, String>;

/// Canonical root type name to its fully resolved semantic type.
pub type NamedTypeMap = BTreeMap<String, Ty>;

/// How a record member call should be lowered by the compiler.
///
/// **Documentation:** `docs/pascal/language/types/record-methods.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodCallTarget {
    /// Instance method: emit the receiver, then the explicit arguments.
    Instance {
        /// Qualified callable name.
        qualified_name: String,
        /// Property getter reads needed while evaluating the receiver designator.
        receiver_reads: Vec<PropertyReadInfo>,
    },
    /// Static record routine: emit only the explicit arguments (no receiver).
    Static(String),
}

impl MethodCallTarget {
    /// Qualified callable name used for `Op::Call` (e.g. `Point.Create`).
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        match self {
            Self::Instance { qualified_name, .. } | Self::Static(qualified_name) => qualified_name,
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

/// Semantic metadata for a bound instance-method value (`C.Add`).
///
/// **Documentation:** `docs/pascal/language/types/record-methods.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundMethodInfo {
    /// Qualified instance method name (e.g. `Counter.Add`).
    pub qualified_name: String,
    /// Explicit argument count after binding (Self omitted).
    pub visible_arity: u8,
    /// Number of designator parts forming the receiver before the method name.
    ///
    /// One for `C.Add`, three for `Items[0].Add`, and zero for postfix `.Add` because its
    /// receiver is already on the stack.
    pub receiver_part_count: usize,
}

/// Maps designator or postfix-operation identity to [`BoundMethodInfo`].
pub type BoundMethodMap = HashMap<usize, BoundMethodInfo>;

/// Semantic metadata for a property getter read (`B.Text`).
///
/// **Documentation:** `docs/pascal/language/types/record-properties.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyReadInfo {
    /// Qualified getter name (e.g. `Button.GetText`).
    pub getter_name: String,
    /// Number of designator parts forming the receiver before the property name.
    ///
    /// Zero for postfix `.Text` because its receiver is already on the stack.
    pub receiver_part_count: usize,
}

/// Maps designator or postfix Field identity to its ordered property getter reads.
pub type PropertyReadMap = HashMap<usize, Vec<PropertyReadInfo>>;

/// Semantic metadata for a property setter assignment (`B.Text := …`).
///
/// **Documentation:** `docs/pascal/language/types/record-properties.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyWriteInfo {
    /// Qualified setter name (e.g. `Button.SetText`).
    pub setter_name: String,
    /// Number of designator parts forming the receiver before the property name.
    pub receiver_part_count: usize,
    /// Ordered property getter reads needed while evaluating the receiver path.
    pub receiver_reads: Vec<PropertyReadInfo>,
}

/// Maps assignment-target designator identity to [`PropertyWriteInfo`].
pub type PropertyWriteMap = HashMap<usize, PropertyWriteInfo>;

/// Semantic metadata for an event setter assignment (`B.OnClick := …` / `:= nil`).
///
/// **Documentation:** `docs/pascal/language/types/record-events.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventWriteInfo {
    /// Qualified setter name (e.g. `Button.WriteOnClick`).
    pub setter_name: String,
    /// Number of designator parts forming the receiver before the event name.
    pub receiver_part_count: usize,
    /// Ordered property getter reads needed while evaluating the receiver path.
    pub receiver_reads: Vec<PropertyReadInfo>,
    /// When `true`, the RHS is `nil` and lowers to `None`; otherwise wrap as `Some`.
    pub clear: bool,
}

/// Maps assignment-target designator identity to [`EventWriteInfo`].
pub type EventWriteMap = HashMap<usize, EventWriteInfo>;

/// Semantic metadata for `Assigned(event)`.
///
/// **Documentation:** `docs/pascal/language/types/record-events.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventAssignedInfo {
    /// Qualified getter name returning `Option of Handler`.
    pub getter_name: String,
    /// Number of designator parts forming the receiver before the event name.
    pub receiver_part_count: usize,
    /// Ordered property getter reads needed while evaluating the receiver path.
    pub receiver_reads: Vec<PropertyReadInfo>,
}

/// Maps `Assigned(...)` call-expression identity to [`EventAssignedInfo`].
pub type EventAssignedMap = HashMap<usize, EventAssignedInfo>;

/// Semantic metadata for owner-only event invocation (`B.OnClick(…)`).
///
/// **Documentation:** `docs/pascal/language/types/record-events.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRaiseInfo {
    /// Qualified getter name returning `Option of Handler`.
    pub getter_name: String,
    /// Number of designator parts forming the receiver before the event name.
    pub receiver_part_count: usize,
    /// Ordered property getter reads needed while evaluating the receiver path.
    pub receiver_reads: Vec<PropertyReadInfo>,
    /// Handler argument count (not counting the event receiver).
    pub arity: u8,
}

/// Maps event call identity (expression or statement designator) to [`EventRaiseInfo`].
pub type EventRaiseMap = HashMap<usize, EventRaiseInfo>;

/// Maps a named record type to its ordered field list, each entry carrying an optional
/// cloned default expression. The order matches the type definition.
///
/// **Documentation:** `docs/pascal/language/types/records.md` (Default field values)
pub type RecordDefaultsMap = HashMap<String, Vec<(String, Option<Expr>)>>;

/// Marks `CaseLabel::Value.start` expressions that semantic analysis interpreted
/// as scalar guard bindings instead of value labels.
pub type ScalarCaseBindingMap = HashSet<usize>;

/// Compiler-facing diagnostics and lowering metadata produced by semantic analysis.
///
/// All identity-keyed maps refer to nodes in the immutable AST passed to the analysis entry
/// point and remain valid only while compiling or inspecting that same AST allocation.
#[derive(Debug, Default)]
pub struct AnalysisMetadata {
    /// Semantic diagnostics. An empty collection means analysis succeeded.
    pub errors: Vec<SemaError>,
    /// Inferred expression types keyed by expression identity.
    pub expr_types: ExprTypeMap,
    /// Canonical standard-library calls keyed by expression or designator identity.
    pub intrinsic_calls: IntrinsicCallMap,
    /// Fully resolved named types used to construct deterministic runtime layouts.
    pub named_types: NamedTypeMap,
    /// Resolved record method calls keyed by expression or designator identity.
    pub method_calls: MethodCallMap,
    /// Named record defaults used while lowering record literals.
    pub record_defaults: RecordDefaultsMap,
    /// Scalar `case` labels interpreted as guard bindings.
    pub scalar_case_bindings: ScalarCaseBindingMap,
    /// Capture metadata for anonymous closures.
    pub closure_infos: ClosureInfoMap,
    /// Capture metadata for escaping named nested routines.
    pub nested_routine_captures: NestedRoutineCaptureMap,
    /// Bound instance-method values keyed by designator identity.
    pub bound_methods: BoundMethodMap,
    /// Property getter reads keyed by designator identity.
    pub property_reads: PropertyReadMap,
    /// Property setter assignments keyed by assignment-target identity.
    pub property_writes: PropertyWriteMap,
    /// Event setter assignments keyed by assignment-target identity.
    pub event_writes: EventWriteMap,
    /// `Assigned(event)` calls keyed by call-expression identity.
    pub event_assigned: EventAssignedMap,
    /// Event raise calls keyed by expression or designator identity.
    pub event_raises: EventRaiseMap,
}

pub struct Checker {
    pub(crate) scopes: ScopeStack,
    pub(crate) errors: Vec<SemaError>,
    pub(crate) expr_types: ExprTypeMap,
    /// Canonical standard-library calls keyed by expression or designator identity.
    pub(crate) intrinsic_calls: IntrinsicCallMap,
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
    /// Closure expression identity → capture / capability metadata.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) closure_infos: ClosureInfoMap,
    /// Nested routine name → capture metadata.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) nested_routine_captures: NestedRoutineCaptureMap,
    /// Designator / postfix Field identity → bound method metadata.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-methods.md`
    pub(crate) bound_methods: BoundMethodMap,
    /// Designator / postfix Field identity → property getter metadata.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub(crate) property_reads: PropertyReadMap,
    /// Assignment-target designator identity → property setter metadata.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub(crate) property_writes: PropertyWriteMap,
    /// Assignment-target designator identity → event setter metadata.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) event_writes: EventWriteMap,
    /// `Assigned(event)` call identity → getter metadata.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) event_assigned: EventAssignedMap,
    /// Event raise call identity → getter / arity metadata.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) event_raises: EventRaiseMap,
    /// Expression keys whose value is a task-bound callable.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) task_bound_exprs: HashSet<usize>,
}

impl Checker {
    pub fn new() -> Self {
        Self {
            scopes: ScopeStack::new(),
            errors: Vec::new(),
            expr_types: ExprTypeMap::new(),
            intrinsic_calls: IntrinsicCallMap::new(),
            method_calls: MethodCallMap::new(),
            loaded_std_units: HashSet::new(),
            ambiguous_imports: HashMap::new(),
            ambiguous_enum_variants: HashMap::new(),
            enum_short_variant_keys: HashMap::new(),
            short_builtin_redirect: HashMap::new(),
            std_short_alias_keys: HashSet::new(),
            record_defaults: RecordDefaultsMap::new(),
            scalar_case_bindings: ScalarCaseBindingMap::new(),
            closure_infos: ClosureInfoMap::new(),
            nested_routine_captures: NestedRoutineCaptureMap::new(),
            bound_methods: BoundMethodMap::new(),
            property_reads: PropertyReadMap::new(),
            property_writes: PropertyWriteMap::new(),
            event_writes: EventWriteMap::new(),
            event_assigned: EventAssignedMap::new(),
            event_raises: EventRaiseMap::new(),
            task_bound_exprs: HashSet::new(),
        }
    }

    pub fn finish(self) -> AnalysisMetadata {
        let named_types = self.scopes.root_types();
        AnalysisMetadata {
            errors: self.errors,
            expr_types: self.expr_types,
            intrinsic_calls: self.intrinsic_calls,
            named_types,
            method_calls: self.method_calls,
            record_defaults: self.record_defaults,
            scalar_case_bindings: self.scalar_case_bindings,
            closure_infos: self.closure_infos,
            nested_routine_captures: self.nested_routine_captures,
            bound_methods: self.bound_methods,
            property_reads: self.property_reads,
            property_writes: self.property_writes,
            event_writes: self.event_writes,
            event_assigned: self.event_assigned,
            event_raises: self.event_raises,
        }
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

    pub(crate) fn mark_expr_task_bound(&mut self, key: usize) {
        self.task_bound_exprs.insert(key);
    }

    pub(crate) fn expr_is_task_bound(&self, key: usize) -> bool {
        self.task_bound_exprs.contains(&key)
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
