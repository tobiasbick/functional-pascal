//! Compiles FPAS AST to bytecode. Lowers `Std.Console` I/O, `Std.Math.Pi`, and `Std.Array` mutating calls, among others.
//!
//! **Documentation:** `docs/pascal/std/console.md`, `docs/pascal/std/math.md`, `docs/pascal/std/array.md` (from the repository root).
//! **Maintenance:** Keep those Markdown files in sync when changing how standard calls are emitted.

use std::collections::HashMap;

use fpas_bytecode::Chunk;
use fpas_sema::{ExprTypeMap, MethodCallMap};

mod binary_op;
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
}

/// Result of resolving a variable name.
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
    /// Record type name → set of method names (for method call dispatch).
    record_methods: HashMap<String, Vec<String>>,
    /// Maps call-expression/designator identity to qualified method name.
    method_calls: MethodCallMap,
    /// Counter for generating unique lambda function names.
    next_lambda_id: u32,
}

struct LoopCtx {
    break_patches: Vec<usize>,
    continue_patches: Vec<usize>,
    scope_depth: u32,
}

impl Compiler {
    pub fn new(expr_types: ExprTypeMap, method_calls: MethodCallMap) -> Self {
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
            record_methods: HashMap::new(),
            method_calls,
            next_lambda_id: 0,
        }
    }

    pub fn finish(self) -> Chunk {
        self.chunk
    }
}
