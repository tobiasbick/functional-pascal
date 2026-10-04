//! Compiler-facing semantic analysis metadata.

use crate::check::closures::{ClosureInfoMap, NestedRoutineCaptureMap};
use crate::error::SemaError;
use crate::types::Ty;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Maps expression identity (`Expr` as `*const Expr`) to its semantic type.
pub type ExprTypeMap = HashMap<usize, Ty>;

/// Resolved binding types keyed by source identity and declaration start offset.
pub type BindingTypeMap = HashMap<(u32, usize), Ty>;

/// Maps a call expression or call-statement designator to its canonical `Std.*` dispatch name.
pub type IntrinsicCallMap = HashMap<usize, String>;

use crate::check::calls::ValueCallMap;

/// Canonical root type name to its fully resolved semantic type.
pub type NamedTypeMap = BTreeMap<String, Ty>;

use super::record_defaults::RecordDefaultsMap;

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
    /// Declared or inferred types of const/var bindings at their source declarations.
    pub binding_types: BindingTypeMap,
    /// Canonical standard-library calls keyed by expression or designator identity.
    pub intrinsic_calls: IntrinsicCallMap,
    /// Fully resolved named types used to construct deterministic runtime layouts.
    pub named_types: NamedTypeMap,
    /// Import alias to canonical linked unit identity.
    pub import_aliases: HashMap<String, String>,
    /// Checked calls through ordinary callable values and record members.
    pub value_calls: ValueCallMap,
    /// Named record defaults used while lowering record constructions.
    pub record_defaults: RecordDefaultsMap,
    /// Expression identities whose call syntax resolves to record construction.
    pub record_constructions: HashSet<usize>,
    /// Concrete result types of field/index projections, keyed by their AST identity.
    pub projection_types: ExprTypeMap,
    /// Concrete types and resolved variants for recursive case patterns.
    pub pattern_infos: crate::check::patterns::PatternInfoMap,
    /// Statement cases with proven complete coverage, keyed by scrutinee identity.
    pub exhaustive_cases: HashSet<usize>,
    /// Capture metadata for anonymous closures.
    pub closure_infos: ClosureInfoMap,
    /// Capture metadata for escaping named nested routines.
    pub nested_routine_captures: NestedRoutineCaptureMap,
}
