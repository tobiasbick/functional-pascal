/// Bytecode instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Push constant from the chunk's constant pool.
    ///
    /// Valid indices are `0..chunk.constants().len()` and never exceed
    /// [`crate::MAX_CONSTANT_INDEX`].
    Constant(u16),

    /// Push Unit value.
    Unit,

    // ── Stack ───────────────────────────────────────────────
    /// Pop top of stack.
    Pop,
    /// Duplicate top of stack.
    Dup,

    // ── Locals ──────────────────────────────────────────────
    /// Load local variable at slot index.
    GetLocal(u16),
    /// Store top-of-stack into local variable slot (does not pop).
    SetLocal(u16),

    // ── Globals ─────────────────────────────────────────────
    /// Load global variable by constant-pool index (name).
    GetGlobal(u16),
    /// Define/store global variable by constant-pool index (name).
    SetGlobal(u16),
    /// Set one element through an all-index path rooted at a global array.
    ///
    /// Stack: `[..., index0, ..., indexN, value]` → `[..., ()]`. The VM updates the global
    /// array under its write lock and applies copy-on-write only to affected shared paths.
    GlobalIndexSet(u16, u8),

    // ── Arithmetic ──────────────────────────────────────────
    AddInt,
    SubInt,
    MulInt,
    DivInt,
    ModInt,

    AddReal,
    SubReal,
    MulReal,
    DivReal,

    NegateInt,
    NegateReal,

    // ── Dynamic numeric (type-erased generics) ──────────────
    /// Polymorphic arithmetic — dispatches at runtime based on value types.
    /// Used inside generic function bodies when compile-time type is GenericParam.
    AddDyn,
    SubDyn,
    MulDyn,
    DivDyn,
    NegateDyn,

    // ── Dynamic comparison (type-erased generics) ───────────
    EqDyn,
    NeqDyn,
    LtDyn,
    GtDyn,
    LeDyn,
    GeDyn,

    // ── String ──────────────────────────────────────────────
    ConcatStr,

    // ── Bitwise / shift ─────────────────────────────────────
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,

    // ── Comparison ──────────────────────────────────────────
    EqInt,
    NeqInt,
    LtInt,
    GtInt,
    LeInt,
    GeInt,

    EqReal,
    NeqReal,
    LtReal,
    GtReal,
    LeReal,
    GeReal,

    EqStr,
    NeqStr,
    LtStr,
    GtStr,
    LeStr,
    GeStr,

    EqBool,
    NeqBool,

    // ── Logical ─────────────────────────────────────────────
    Not,
    /// Boolean and (both operands already evaluated — no short-circuit at opcode level).
    And,
    Or,

    // ── Type conversion ─────────────────────────────────────
    IntToReal,

    // ── Control flow ────────────────────────────────────────
    /// Unconditional jump (absolute offset in code).
    Jump(u32),
    /// Pop top; jump if false.
    JumpIfFalse(u32),
    /// Pop top; jump if true.
    JumpIfTrue(u32),

    // ── Functions ───────────────────────────────────────────
    /// Call function/procedure at constant-pool index (name), with arg_count args on stack.
    Call(u16, u8),
    /// Call a function value on top of the stack with arg_count args below it.
    ///
    /// Stack layout: `[..., arg0, arg1, ..., argN, function_value]` → `[..., return_value]`
    ///
    /// **Documentation:** `docs/pascal/language/functions/first-class.md`
    CallValue(u8),
    /// Build a closure from `capture_count` values below a name constant.
    ///
    /// Stack: `[..., capture0, …, captureN]` → `[..., Function { name, captures, task_bound }]`.
    /// `task_bound` is true when any capture is a [`crate::Value::Cell`].
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    MakeClosure(u16, u8),
    /// Wrap the top-of-stack value in a shared mutable [`crate::Value::Cell`].
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    MakeCell,
    /// Read the value inside a [`crate::Value::Cell`] on top of the stack.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    CellGet,
    /// Pop value, then pop cell; store value into the cell; push unit.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    CellSet,
    /// Return from function (top-of-stack is return value).
    Return,
    /// Read a local from an enclosing function's call frame.
    /// (call-stack depth, slot) — depth 1 = immediate parent.
    GetEnclosing(u16, u16),
    /// Write a local in an enclosing function's call frame.
    /// (call-stack depth, slot) — depth 1 = immediate parent.
    SetEnclosing(u16, u16),

    // ── Arrays ──────────────────────────────────────────────
    /// Build array from N elements on stack.
    MakeArray(u16),
    /// Index into array or dict: [collection, index/key] → value.
    IndexGet,
    /// Index/key set: [collection, index/key, value] → ().
    IndexSet,
    /// Membership test: [value, collection] → boolean.
    ///
    /// Supports arrays (element membership), dicts (key membership), and strings
    /// (character or substring membership).
    ///
    /// **Documentation:** `docs/pascal/language/basics/operators.md`, `docs/specs/grammar.ebnf`
    Contains,

    // ── Dicts ───────────────────────────────────────────────
    /// Build dict from N key-value pairs on stack (2*N values: k0, v0, k1, v1, …).
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionaries.md`
    MakeDict(u16),

    // ── Records ─────────────────────────────────────────────
    /// Build record: N field values on stack + constant index for type name.
    MakeRecord(u16, u16),
    /// Get field by constant-pool index (field name): [record] → value.
    FieldGet(u16),
    /// Set field by constant-pool index (field name): [record, value] → ().
    FieldSet(u16),
    /// Copy a record and override N fields: [base, name0, val0, …, nameN-1, valN-1] → record.
    ///
    /// The operand is the number of (name, value) override pairs.
    /// All non-overridden fields are copied from the base record unchanged.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-update.md`
    UpdateRecord(u16),

    // ── Special ─────────────────────────────────────────────
    /// Print value (for WriteLn/Write builtins).
    Print,
    /// Print value + newline.
    PrintLn,
    /// Built-in intrinsic (`fpas_bytecode::intrinsic::Intrinsic` as u16).
    Intrinsic(u16),
    /// Append to array held in local `(enclosing_depth, slot)`; value on stack top.
    ArrayPushLocal(u16, u16),
    /// Pop from array in local `(enclosing_depth, slot)`; pushes removed element.
    ArrayPopLocal(u16, u16),
    /// Halt execution.
    Halt,
    /// Runtime panic with string message on stack.
    Panic,

    // ── Result / Option ─────────────────────────────────────
    /// Wrap top-of-stack into `ResultOk`.
    MakeOk,
    /// Wrap top-of-stack into `ResultError`.
    MakeErr,
    /// Wrap top-of-stack into `OptionSome`.
    MakeSome,
    /// Push `OptionNone`.
    MakeNone,
    /// Pop Result; push `true` if `ResultOk`, `false` if `ResultError`.
    IsResultOk,
    /// Pop Option; push `true` if `OptionSome`, `false` if `OptionNone`.
    IsOptionSome,
    /// Pop `ResultOk(v)` → push `v`; panics on `ResultErr`.
    UnwrapOk,
    /// Pop `ResultError(e)` → push `e`; panics on `ResultOk`.
    UnwrapErr,
    /// Pop `OptionSome(v)` → push `v`; panics on `OptionNone`.
    UnwrapSome,

    // ── Enums with Associated Data ──────────────────────────
    /// Build enum variant: field_count values on stack.
    /// (type_name_const_idx, variant_name_const_idx, field_count)
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    MakeEnum(u16, u16, u8),
    /// Pop enum value; push `true` if variant name matches the constant at index.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    IsVariant(u16),
    /// Pop enum value; push field at given positional index. Panics on wrong variant.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    EnumField(u8),

    // ── Concurrency (stack VM task surface) ─────────────────
    /// Spawn a new task: pops function value + `arg_count` args, pushes `Value::Task(id)`.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 1), `docs/pascal/language/concurrency/README.md`
    SpawnTask(u8),
    /// Spawn a detached task: pops function value + `arg_count` args and discards its result.
    ///
    /// Used for statement-form `go` where no task handle is retained.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 1), `docs/pascal/language/concurrency/README.md`
    SpawnDetachedTask(u8),
    /// Yield execution to the scheduler (cooperative switch).
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 1), `docs/pascal/language/concurrency/README.md`
    Yield,
}
