//! Checker state and shared diagnostics.

mod metadata;
mod record_defaults;

use crate::check::calls::ValueCallMap;
pub use metadata::*;
pub use record_defaults::*;

use crate::check::closures::{ClosureInfoMap, NestedRoutineCaptureMap};
use crate::error::{SemaError, sema_error};
use crate::scope::ScopeStack;
use fpas_diagnostics::DiagnosticCode;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::{HashMap, HashSet};

pub struct Checker {
    pub(in crate::check) pure_evaluation: Option<crate::check::purity::PureEvaluation>,
    pub(in crate::check) pure_parameters: HashSet<crate::types::GenericParameterId>,
    pub(in crate::check) default_purity: crate::check::purity::defaults::DefaultPurity,
    pub(in crate::check) type_collection: crate::check::decl::types::collection::TypeCollection,
    pub(crate) scopes: ScopeStack,
    pub(crate) errors: Vec<SemaError>,
    pub(crate) expr_types: ExprTypeMap,
    pub(crate) binding_types: BindingTypeMap,
    /// Canonical standard-library calls keyed by expression or designator identity.
    pub(crate) intrinsic_calls: IntrinsicCallMap,
    /// Checked calls through ordinary callable values and record members.
    pub(crate) value_calls: ValueCallMap,
    pub(crate) import_aliases: HashMap<String, String>,
    /// Source spelling of each import alias, keyed by its canonical lowercase name.
    pub(crate) import_alias_spellings: HashMap<String, String>,
    pub(crate) supporting_unit_names: HashSet<String>,
    /// Argument expressions already checked with their contextual callback signatures.
    pub(crate) prechecked_arguments: ExprTypeMap,
    /// Canonical std unit names from `uses` (e.g. `Std.Console`).
    pub(crate) loaded_std_units: HashSet<String>,
    /// All unit names in the current `uses` clause, including source units.
    pub(crate) used_unit_names: HashSet<String>,
    /// Named record type → ordered (field_name, optional_default_expr) pairs.
    pub(crate) record_defaults: RecordDefaultsMap,
    pub(crate) record_constructions: HashSet<usize>,
    pub(crate) projection_types: ExprTypeMap,
    pub(crate) pattern_infos: crate::check::patterns::PatternInfoMap,
    pub(crate) exhaustive_cases: HashSet<usize>,
    pub(crate) static_constants: crate::interface::StaticConstants,
    /// Nesting of initializer groups whose remaining values may supply type context.
    pub(crate) inference_depth: usize,
    /// Closure expression identity → capture / capability metadata.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) closure_infos: ClosureInfoMap,
    /// Nested routine name → capture metadata.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) nested_routine_captures: NestedRoutineCaptureMap,
    /// Expression keys whose value is a task-bound callable.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) task_bound_exprs: HashSet<usize>,
}

impl Checker {
    pub fn new() -> Self {
        Self {
            pure_evaluation: None,
            pure_parameters: HashSet::new(),
            default_purity: crate::check::purity::defaults::DefaultPurity::default(),
            type_collection: crate::check::decl::types::collection::TypeCollection::default(),
            scopes: ScopeStack::new(),
            errors: Vec::new(),
            expr_types: ExprTypeMap::new(),
            binding_types: BindingTypeMap::new(),
            intrinsic_calls: IntrinsicCallMap::new(),
            value_calls: ValueCallMap::new(),
            import_aliases: HashMap::new(),
            import_alias_spellings: HashMap::new(),
            supporting_unit_names: HashSet::new(),
            prechecked_arguments: ExprTypeMap::new(),
            loaded_std_units: HashSet::new(),
            used_unit_names: HashSet::new(),
            record_defaults: RecordDefaultsMap::new(),
            record_constructions: HashSet::new(),
            projection_types: ExprTypeMap::new(),
            pattern_infos: crate::check::patterns::PatternInfoMap::new(),
            exhaustive_cases: HashSet::new(),
            static_constants: crate::interface::StaticConstants::default(),
            inference_depth: 0,
            closure_infos: ClosureInfoMap::new(),
            nested_routine_captures: NestedRoutineCaptureMap::new(),
            task_bound_exprs: HashSet::new(),
        }
    }

    pub fn finish(mut self) -> AnalysisMetadata {
        self.report_import_alias_conflicts();
        let named_types = self.scopes.root_types();
        AnalysisMetadata {
            errors: self.errors,
            expr_types: self.expr_types,
            binding_types: self.binding_types,
            intrinsic_calls: self.intrinsic_calls,
            named_types,
            import_aliases: self.import_aliases,
            value_calls: self.value_calls,
            record_defaults: self.record_defaults,
            record_constructions: self.record_constructions,
            projection_types: self.projection_types,
            pattern_infos: self.pattern_infos,
            exhaustive_cases: self.exhaustive_cases,
            closure_infos: self.closure_infos,
            nested_routine_captures: self.nested_routine_captures,
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
