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

/// Maps the first written argument of a named call to the written argument
/// index for each parameter, in parameter order.
///
/// **Documentation:** `docs/pascal/language/functions/parameters.md`
pub type NamedArgumentOrderMap = HashMap<usize, Vec<usize>>;

/// Fixed native catalog target selected for a built-in dot call.
///
/// **Documentation:** `docs/pascal/language/functions/fluent-calls.md`
#[derive(Debug, Clone, PartialEq)]
pub struct FluentCallTarget {
    /// Private implementation identity of the selected native operation.
    pub name: String,
    /// Static receiver type selecting the catalog operation.
    pub receiver_ty: Ty,
    /// Fully checked result type, including generic substitution.
    pub result_ty: Ty,
    /// Source span of the selected call name or postfix operation.
    pub call_span: Span,
}

/// Maps call or postfix-operation identity to its selected receiver-call target.
pub type FluentCallMap = HashMap<usize, FluentCallTarget>;

/// Maps a callable field or indexed value call to its checked result type.
pub type MemberValueCallMap = HashMap<usize, Ty>;

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
    },
    /// Static record routine: emit only the explicit arguments (no receiver).
    Static(String),
}

impl MethodCallTarget {
    /// Qualified callable name used for calls (for example, `Point.Create`).
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
/// Ordinary [`Expr::Call`] entries use [`crate::expr_lookup_key`]
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

/// Maps a named record type to its ordered field list, each entry carrying an optional
/// shared default expression. Stable allocation retains checked expression identity;
/// the order matches the type definition.
///
/// **Documentation:** `docs/pascal/language/types/records.md` (Default field values)
pub type RecordDefaultsMap = HashMap<String, Vec<(String, Option<std::sync::Arc<Expr>>)>>;

/// Compiler-facing diagnostics and lowering metadata produced by semantic analysis.
///
/// Identity-keyed maps refer to the immutable source AST or the retained expressions in
/// `record_defaults`. They remain valid while inspecting those same allocations.
#[derive(Debug, Default)]
pub struct AnalysisMetadata {
    /// Canonical direct unit identities to source-local aliases.
    /// **Documentation:** `docs/pascal/program-structure/units.md`
    pub import_aliases: BTreeMap<String, String>,
    /// Semantic diagnostics. An empty collection means analysis succeeded.
    pub errors: Vec<SemaError>,
    /// Inferred expression types keyed by expression identity.
    pub expr_types: ExprTypeMap,
    /// Designators resolved to enum members, keyed by full designator identity.
    /// **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`
    pub enum_members: HashSet<usize>,
    /// Canonical standard-library calls keyed by expression or designator identity.
    pub intrinsic_calls: IntrinsicCallMap,
    /// Parameter order of named calls keyed by the first written argument's identity.
    pub named_argument_orders: NamedArgumentOrderMap,
    /// Calls resolved to record types, rather than routines returning records.
    /// **Documentation:** `docs/pascal/language/types/records.md`
    pub record_constructions: HashSet<usize>,
    /// Calls that convert into or out of a distinct type; they lower to their argument.
    /// **Documentation:** `docs/pascal/language/types/distinct-types.md`
    pub distinct_conversions: HashSet<usize>,
    /// Fully resolved named types used to construct deterministic runtime layouts.
    pub named_types: NamedTypeMap,
    /// Resolved record method calls keyed by expression or designator identity.
    pub method_calls: MethodCallMap,
    /// Selected fixed built-in dot operations.
    pub fluent_calls: FluentCallMap,
    /// Calls through callable record fields.
    pub member_value_calls: MemberValueCallMap,
    /// Named record defaults used while lowering record constructions.
    pub record_defaults: RecordDefaultsMap,
    /// Capture metadata for anonymous closures.
    pub closure_infos: ClosureInfoMap,
    /// Capture metadata for escaping named nested routines.
    pub nested_routine_captures: NestedRoutineCaptureMap,
    /// Bound instance-method values keyed by designator identity.
    pub bound_methods: BoundMethodMap,
}

/// Semantic checking state for whole-unit types and ordered executable declarations.
pub struct Checker {
    /// Whole-unit structural type resolution, separate from ordered value checking.
    pub(crate) type_collection: super::decl::types::collection::TypeCollection,
    pub(crate) discard_exprs: HashMap<usize, fpas_unit::interface::DiscardInfo>,
    pub(crate) discard_results: Vec<bool>,
    pub(crate) routine_discard_results: HashMap<String, bool>,
    pub(crate) scopes: ScopeStack,
    pub(crate) errors: Vec<SemaError>,
    pub(crate) expr_types: ExprTypeMap,
    /// Full designators whose resolved symbol is an enum member.
    pub(crate) enum_members: HashSet<usize>,
    /// Canonical standard-library calls keyed by expression or designator identity.
    pub(crate) intrinsic_calls: IntrinsicCallMap,
    /// Parameter order of named calls keyed by the first written argument's identity.
    pub(crate) named_argument_orders: NamedArgumentOrderMap,
    pub(crate) method_calls: MethodCallMap,
    /// Selected fixed built-in dot operations.
    pub(crate) fluent_calls: FluentCallMap,
    /// Calls through callable record fields.
    pub(crate) member_value_calls: MemberValueCallMap,
    /// Imported public symbols grouped by their unqualified name.
    pub(crate) imported_candidates: HashMap<String, Vec<String>>,
    /// Synthetic receiver expressions already checked as part of a fluent call.
    pub(crate) prechecked_receivers: ExprTypeMap,
    /// Canonical std unit names from `uses` (e.g. `Std.Console`).
    pub(crate) loaded_std_units: HashSet<String>,
    /// All unit names in the current `uses` clause, including source units.
    pub(crate) used_unit_names: HashSet<String>,
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
    /// Canonical short names currently bound to imported source-unit symbols.
    pub(crate) source_short_alias_keys: HashSet<String>,
    /// Short-name candidates exported by directly imported source units, keyed canonically.
    ///
    /// [`crate::std_registry::register_short_aliases`] merges them with `Std.*` candidates.
    pub(crate) source_short_candidates: HashMap<String, Vec<(String, crate::scope::Symbol)>>,
    /// Named record type → ordered (field_name, optional_default_expr) pairs.
    pub(crate) record_defaults: RecordDefaultsMap,
    /// Identity keys of calls that construct a concrete record type.
    pub(crate) record_constructions: HashSet<usize>,
    /// Identity keys of valid conversions into or out of a distinct type.
    pub(crate) distinct_conversions: HashSet<usize>,
    /// Canonical record and field names → task-freedom of their checked default values.
    pub(crate) record_default_discard: HashMap<(String, String), bool>,
    /// Default expression identity → constant classification in the declaration environment.
    pub(crate) record_default_constants: HashMap<usize, bool>,
    /// Known default fields evaluated in their original declaration environment.
    pub(crate) record_default_values:
        HashMap<usize, Option<fpas_unit::interface::RecordConstantField>>,
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
    /// Expression keys whose value is a task-bound callable.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) task_bound_exprs: HashSet<usize>,
    /// Nested routine declaration identities → the enclosing `var` parameter they use.
    ///
    /// **Documentation:** `docs/pascal/language/functions/var-parameters.md`
    pub(crate) var_parameter_routines: HashMap<usize, String>,
    /// Routine value/task uses awaiting the referenced routine's capture analysis.
    /// See `docs/pascal/language/functions/var-parameters.md`.
    pub(crate) pending_var_parameter_uses:
        HashMap<usize, Vec<super::references::PendingRoutineUse>>,
    /// Named routine dependencies whose capture analysis was active when used.
    /// See `docs/pascal/language/functions/closures.md`.
    pub(crate) pending_routine_captures: HashMap<usize, Vec<usize>>,
    /// Capture propagation for recursive targets that finish after their consumers.
    pub(crate) capture_dependencies: super::closures::CaptureDependencies,
    /// Dictionary key types and their source spans, validated once all types are complete.
    pub(crate) pending_dictionary_keys: Vec<super::name_resolution::PendingKey>,
}

impl Checker {
    /// Creates an empty semantic checker and its expression-analysis tables.
    pub fn new() -> Self {
        Self {
            type_collection: Default::default(),
            discard_exprs: HashMap::new(),
            discard_results: Vec::new(),
            routine_discard_results: HashMap::new(),
            scopes: ScopeStack::new(),
            errors: Vec::new(),
            expr_types: ExprTypeMap::new(),
            enum_members: HashSet::new(),
            intrinsic_calls: IntrinsicCallMap::new(),
            named_argument_orders: NamedArgumentOrderMap::new(),
            method_calls: MethodCallMap::new(),
            fluent_calls: FluentCallMap::new(),
            member_value_calls: MemberValueCallMap::new(),
            imported_candidates: HashMap::new(),
            prechecked_receivers: ExprTypeMap::new(),
            loaded_std_units: HashSet::new(),
            used_unit_names: HashSet::new(),
            ambiguous_imports: HashMap::new(),
            ambiguous_enum_variants: HashMap::new(),
            enum_short_variant_keys: HashMap::new(),
            short_builtin_redirect: HashMap::new(),
            std_short_alias_keys: HashSet::new(),
            source_short_alias_keys: HashSet::new(),
            source_short_candidates: HashMap::new(),
            record_defaults: RecordDefaultsMap::new(),
            record_constructions: HashSet::new(),
            distinct_conversions: HashSet::new(),
            record_default_discard: HashMap::new(),
            record_default_constants: HashMap::new(),
            record_default_values: HashMap::new(),
            closure_infos: ClosureInfoMap::new(),
            nested_routine_captures: NestedRoutineCaptureMap::new(),
            bound_methods: BoundMethodMap::new(),
            task_bound_exprs: HashSet::new(),
            var_parameter_routines: HashMap::new(),
            pending_var_parameter_uses: HashMap::new(),
            pending_routine_captures: HashMap::new(),
            capture_dependencies: Default::default(),
            pending_dictionary_keys: Vec::new(),
        }
    }

    /// Return lowering metadata while retaining the checked default expression allocations.
    pub fn finish(self) -> AnalysisMetadata {
        let named_types = self.scopes.root_types();
        AnalysisMetadata {
            import_aliases: self.scopes.imports.aliases.clone(),
            errors: self.errors,
            expr_types: self.expr_types,
            enum_members: self.enum_members,
            intrinsic_calls: self.intrinsic_calls,
            named_argument_orders: self.named_argument_orders,
            named_types,
            method_calls: self.method_calls,
            fluent_calls: self.fluent_calls,
            member_value_calls: self.member_value_calls,
            record_defaults: self.record_defaults,
            record_constructions: self.record_constructions,
            distinct_conversions: self.distinct_conversions,
            closure_infos: self.closure_infos,
            nested_routine_captures: self.nested_routine_captures,
            bound_methods: self.bound_methods,
        }
    }

    /// Stable identity key for an AST expression node.
    ///
    /// Uses the memory address of the `Expr` reference. This is sound because:
    /// - The source AST is immutable for the entire analysis.
    /// - Checked defaults are retained in shared allocations in `record_defaults`.
    /// - Checked nodes are not moved or cloned; keys belong to those allocations.
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
