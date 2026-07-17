//! Compiles FPAS AST to bytecode. Lowers `Std.Console` I/O, `Std.Math.Pi`, and `Std.Array` mutating calls, among others.
//!
//! **Documentation:** `docs/pascal/std/console/README.md`, `docs/pascal/std/numeric/math.md`, `docs/pascal/std/collections/array/README.md` (from the repository root).
//! **Maintenance:** Keep those Markdown files in sync when changing how standard calls are emitted.

use std::collections::{HashMap, HashSet};

use fpas_bytecode::Chunk;
use fpas_sema::{
    ClosureInfoMap, ExprTypeMap, MethodCallMap, NestedRoutineCaptureMap, RecordDefaultsMap,
    ScalarCaseBindingMap,
};

mod binary_op;
mod closures;
mod designator;
mod emit;
mod expr;
mod locals;
mod program;
mod std_aliases;
mod std_calls;
mod stmt;

/// Tracks a local variable's stack slot.
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: u32,
    slot: u16,
    is_cell: bool,
}

/// Result of resolving a variable name.
#[derive(Clone, Copy)]
enum LocalRef {
    /// Local in the current function frame.
    Local(u16),
    /// Local in an enclosing function frame (depth, slot).
    Enclosing(u16, u16),
}

/// Info about each variant in a registered enum.
#[derive(Debug, Clone)]
struct EnumVariantInfo {
    name: String,
    backing: i64,
    field_names: Vec<String>,
}

/// Info about a registered enum type.
#[derive(Debug, Clone)]
struct EnumInfo {
    variants: Vec<EnumVariantInfo>,
    /// True when at least one variant carries associated data.
    has_data: bool,
}

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: u32,
    next_slot: u16,
    /// Loop context for break/continue: (loop_start, break_patches).
    loop_stack: Vec<LoopCtx>,
    /// Enum type name → variant info.
    enums: HashMap<String, EnumInfo>,
    /// Stack of saved parent locals for nested function variable capture.
    enclosing_locals: Vec<Vec<Local>>,
    /// Short (unqualified) name → fully-qualified `Std.*` name.
    short_aliases: HashMap<String, String>,
    expr_types: ExprTypeMap,
    /// Maps call-expression/designator identity to qualified method name.
    method_calls: MethodCallMap,
    /// Named record type → ordered (field_name, optional_default_expr) pairs.
    /// Used to expand record literals when fields with defaults are omitted.
    ///
    /// **Documentation:** `docs/pascal/language/types/records.md` (Default field values)
    record_defaults: RecordDefaultsMap,
    /// Scalar `case` labels that sema resolved as guard bindings.
    scalar_case_bindings: ScalarCaseBindingMap,
    /// Capture metadata for anonymous closures.
    closure_infos: ClosureInfoMap,
    /// Capture metadata for escaping named nested routines.
    nested_routine_captures: NestedRoutineCaptureMap,
    /// Canonical names of module-level globals (`const` / `var` / `mutable var`).
    module_globals: HashSet<String>,
}

struct LoopCtx {
    break_patches: Vec<usize>,
    continue_patches: Vec<usize>,
    scope_depth: u32,
}

fn canonical_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

impl Compiler {
    /// Create a new compiler with the given sema results.
    pub fn new(
        expr_types: ExprTypeMap,
        method_calls: MethodCallMap,
        record_defaults: RecordDefaultsMap,
        scalar_case_bindings: ScalarCaseBindingMap,
        closure_infos: ClosureInfoMap,
        nested_routine_captures: NestedRoutineCaptureMap,
    ) -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            next_slot: 0,
            loop_stack: Vec::new(),
            enums: HashMap::new(),
            enclosing_locals: Vec::new(),
            short_aliases: HashMap::new(),
            expr_types,
            method_calls,
            record_defaults,
            scalar_case_bindings,
            closure_infos,
            nested_routine_captures,
            module_globals: HashSet::new(),
        }
    }

    pub fn finish(self) -> Chunk {
        self.chunk
    }
}
