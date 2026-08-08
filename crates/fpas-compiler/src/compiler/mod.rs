//! Compiles FPAS AST to bytecode. Lowers `Std.Console` I/O, `Std.Math.Pi`, and `Std.Array` mutating calls, among others.
//!
//! **Documentation:** `docs/pascal/std/console/README.md`, `docs/pascal/std/numeric/math.md`, `docs/pascal/std/collections/array/README.md` (from the repository root).
//! **Maintenance:** Keep those Markdown files in sync when changing how standard calls are emitted.

use std::collections::{HashMap, HashSet};

use fpas_bytecode::Chunk;
use fpas_sema::{
    AnalysisMetadata, BoundMethodMap, ClosureInfoMap, EventAssignedMap, EventRaiseMap,
    EventWriteMap, ExprTypeMap, MethodCallMap, NestedRoutineCaptureMap, PropertyReadMap,
    PropertyWriteMap, RecordDefaultsMap, ScalarCaseBindingMap,
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
    /// Declaration name stored on `MakeEnum` values (not a linked/qualified spelling).
    type_name: String,
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
    /// Canonical names of imported callables that may be used as first-class values.
    external_callables: HashSet<String>,
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
    /// Bound instance-method values (`C.Add`).
    bound_methods: BoundMethodMap,
    /// Property getter reads (`B.Text`).
    property_reads: PropertyReadMap,
    /// Property setter assignments (`B.Text := …`).
    property_writes: PropertyWriteMap,
    /// Event setter assignments (`B.OnClick := …`).
    event_writes: EventWriteMap,
    /// `Assigned(event)` calls.
    event_assigned: EventAssignedMap,
    /// Event raise calls.
    event_raises: EventRaiseMap,
    /// Canonical names of module-level globals (`const` / `var` / `mutable var`).
    module_globals: HashSet<String>,
    /// Canonical unit prefix while compiling an independently analyzed unit.
    owner_unit: Option<String>,
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
    pub fn new(metadata: AnalysisMetadata) -> Self {
        let AnalysisMetadata {
            errors: _,
            expr_types,
            intrinsic_calls: _,
            named_types: _,
            method_calls,
            record_defaults,
            scalar_case_bindings,
            closure_infos,
            nested_routine_captures,
            bound_methods,
            property_reads,
            property_writes,
            event_writes,
            event_assigned,
            event_raises,
        } = metadata;
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            next_slot: 0,
            loop_stack: Vec::new(),
            enums: HashMap::new(),
            enclosing_locals: Vec::new(),
            short_aliases: HashMap::new(),
            external_callables: HashSet::new(),
            expr_types,
            method_calls,
            record_defaults,
            scalar_case_bindings,
            closure_infos,
            nested_routine_captures,
            bound_methods,
            property_reads,
            property_writes,
            event_writes,
            event_assigned,
            event_raises,
            module_globals: HashSet::new(),
            owner_unit: None,
        }
    }

    fn set_owner_unit(&mut self, owner: &str) {
        self.owner_unit = Some(owner.to_ascii_lowercase());
    }

    fn qualify_owned_name(&self, name: &str) -> String {
        let Some(owner) = &self.owner_unit else {
            return name.to_string();
        };
        if name.to_ascii_lowercase().starts_with(&format!("{owner}.")) {
            name.to_string()
        } else {
            format!("{owner}.{name}")
        }
    }

    pub fn finish(self) -> Chunk {
        self.chunk
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::Compiler;
    use fpas_sema::{
        AnalysisMetadata, BoundMethodInfo, ClosureInfo, EventAssignedInfo, EventRaiseInfo,
        EventWriteInfo, MethodCallTarget, NestedRoutineCaptureInfo, PropertyReadInfo,
        PropertyWriteInfo, Ty,
    };

    #[test]
    fn compiler_receives_every_lowering_metadata_map() {
        let mut metadata = AnalysisMetadata::default();
        metadata.expr_types.insert(1, Ty::Integer);
        metadata
            .method_calls
            .insert(2, MethodCallTarget::Static("Record.Make".into()));
        metadata.record_defaults.insert("Record".into(), Vec::new());
        metadata.scalar_case_bindings.insert(4);
        metadata.closure_infos.insert(
            5,
            ClosureInfo {
                captures: Vec::new(),
                task_bound: false,
                synthetic_name: "Closure".into(),
            },
        );
        metadata.nested_routine_captures.insert(
            "Nested".into(),
            NestedRoutineCaptureInfo {
                captures: Vec::new(),
                task_bound: false,
            },
        );
        metadata.bound_methods.insert(
            7,
            BoundMethodInfo {
                qualified_name: "Record.Method".into(),
                visible_arity: 0,
                receiver_part_count: 1,
            },
        );
        metadata.property_reads.insert(
            8,
            vec![PropertyReadInfo {
                getter_name: "Record.Get".into(),
                receiver_part_count: 1,
            }],
        );
        metadata.property_writes.insert(
            9,
            PropertyWriteInfo {
                setter_name: "Record.Set".into(),
                receiver_part_count: 1,
                receiver_reads: Vec::new(),
            },
        );
        metadata.event_writes.insert(
            10,
            EventWriteInfo {
                setter_name: "Record.WriteEvent".into(),
                receiver_part_count: 1,
                receiver_reads: Vec::new(),
                clear: false,
            },
        );
        metadata.event_assigned.insert(
            11,
            EventAssignedInfo {
                getter_name: "Record.ReadEvent".into(),
                receiver_part_count: 1,
                receiver_reads: Vec::new(),
            },
        );
        metadata.event_raises.insert(
            12,
            EventRaiseInfo {
                getter_name: "Record.ReadEvent".into(),
                receiver_part_count: 1,
                receiver_reads: Vec::new(),
                arity: 1,
            },
        );

        let compiler = Compiler::new(metadata);

        assert!(
            compiler.expr_types.contains_key(&1)
                && compiler.method_calls.contains_key(&2)
                && compiler.record_defaults.contains_key("Record")
                && compiler.scalar_case_bindings.contains(&4)
                && compiler.closure_infos.contains_key(&5)
                && compiler.nested_routine_captures.contains_key("Nested")
                && compiler.bound_methods.contains_key(&7)
                && compiler.property_reads.contains_key(&8)
                && compiler.property_writes.contains_key(&9)
                && compiler.event_writes.contains_key(&10)
                && compiler.event_assigned.contains_key(&11)
                && compiler.event_raises.contains_key(&12)
        );
    }
}
