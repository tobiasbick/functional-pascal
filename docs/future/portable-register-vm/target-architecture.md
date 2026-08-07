# Target architecture

## Architectural invariants

The target is a safe Rust register interpreter over a fully validated executable. Validation occurs
once when building or loading an artifact. The hot loop may rely on validated structural invariants,
but it must remain memory safe without `unsafe` indexing.

The runtime consists of four deliberately separate layers:

```text
Typed IR             compiler-owned meaning and control flow
Register bytecode    compact portable execution contract
Executable metadata  functions, constants, globals, layouts, strings, sparse locations
VM state             register frames, globals, tasks, hosts, output, diagnostics
```

Do not merge these layers into one large `Chunk` replacement.

## Target crate ownership

The final file layout is fixed at the responsibility level. Exact private helper names may vary when
the checkout requires it, but concerns must remain in the listed modules and files should normally
stay below 500 lines.

```text
crates/fpas-ir/                              NEW crate: target-independent typed IR
  src/lib.rs                                module declarations and public re-exports only
  src/id.rs                                 FunctionId, BlockId, ValueId, LocalId, TypeId
  src/program.rs                            Program and deterministic tables
  src/function.rs                           Function, signature, locals, blocks
  src/instruction.rs                        typed three-address operations
  src/terminator.rs                         branch, jump, return, panic terminators
  src/validate/mod.rs                       validation coordinator
  src/validate/control_flow.rs              block and terminator invariants
  src/validate/operands.rs                  ID, type, and definition invariants

crates/fpas-bytecode/src/
  lib.rs                                    focused exports and BYTECODE_VERSION
  instruction.rs                            packed Instruction(u64), Opcode, safe codecs
  operand.rs                                Register and numeric ID newtypes
  executable.rs                             Executable aggregate and top-level validation
  function.rs                               FunctionInfo and CodeRange
  metadata/
    mod.rs                                  metadata exports
    constants.rs                            persistent constant values
    globals.rs                              global declarations
    records.rs                              record layouts and field slots
    enums.rs                                enum and variant layouts
    strings.rs                              deterministic interned strings
    source_map.rs                           sparse source runs and lookup
  validate/
    mod.rs                                  verifier entry point
    instruction.rs                          opcode-form and operand checks
    control_flow.rs                         function-local targets and reachability
    calls.rs                                arity and call-window checks
    layouts.rs                              type, field, and variant checks
  value/                                    retained runtime value concern modules

crates/fpas-compiler/src/
  lowering/                                 NEW AST + sema -> fpas-ir implementation
    mod.rs                                  lowering entry point
    context.rs                              symbol and scope state
    program.rs                              declarations and initialization order
    callable.rs                             functions, procedures, methods
    expr/                                   expressions by concern
    stmt/                                   statements and control flow by concern
    closure.rs                              capture and cell lowering
    aggregate.rs                            array, dict, record, enum lowering
    intrinsic/                              Std.* lowering by unit/theme
  bytecode/                                 NEW fpas-ir -> register bytecode implementation
    mod.rs                                  code generation entry point
    allocation.rs                           deterministic register allocation
    blocks.rs                               layout and jump patching
    instruction.rs                         typed IR instruction selection
    calls.rs                                contiguous argument windows
    metadata.rs                             constants, symbols, layouts, source runs
  unit_object.rs                            emit the new relocatable object shape

crates/fpas-unit/src/object/
  mod.rs                                    relocatable object aggregate
  symbol.rs                                 imports, exports, definitions
  function.rs                               relocatable function bodies
  metadata.rs                               strings, globals, and type layouts
  relocation.rs                             symbolic references and local targets
  format/                                   bounded deterministic `.fpascu` codec

crates/fpas-linker/src/
  lib.rs                                    orchestration only
  symbols.rs                                deterministic symbol collection and ID assignment
  functions.rs                              function table and code layout
  globals.rs                                global slot assignment
  layouts.rs                                record/enum layout unification
  constants.rs                              deterministic constant merge
  relocation.rs                             rewrite symbolic operands to numeric IDs
  source_map.rs                             source-table merge and run rebasing
  validate.rs                               final executable validation handoff

crates/fpas-program/src/
  image.rs                                  identity + register Executable
  format/
    mod.rs                                  format version and public codec
    header.rs                               magic, versions, sizes, payload digest
    sections.rs                             section tags and directory validation
    read.rs                                 bounded explicit little-endian decoder
    write.rs                                deterministic explicit little-endian encoder
    executable.rs                           executable section conversion

crates/fpas-vm/src/vm/
  mod.rs                                    public VM lifecycle only
  worker.rs                                 per-thread execution state
  frame.rs                                  register-window call frame
  dispatch.rs                               one exhaustive top-level opcode match
  execute/                                  direct operand handlers by concern
  values/                                   runtime value access and formatting context
  diagnostics.rs                            instruction address -> sparse source lookup
  shared/                                   globals, tasks, timers, hosts
```

At cutover, remove superseded stack modules rather than leaving `legacy`, `old`, or `v1` directories.
Do not create a generic `utils.rs` or `helpers.rs`; migrate remaining helpers to the concern that owns
them.

## Typed IR

The IR is a typed, non-serialized, target-independent control-flow graph. It is not FPAS syntax and it
is not the persistent bytecode format.

Each `Function` contains:

- `FunctionId` and canonical diagnostic name;
- parameter and result types;
- ordered local declarations with mutability and capture information;
- basic blocks in deterministic reverse-postorder;
- instructions whose operands are `ValueId` or `LocalId`;
- exactly one terminator per block;
- source spans attached to semantic operations, not to synthetic moves;
- maximum call argument count and whether the function can spawn tasks.

Use explicit operations such as `Const`, `ReadLocal`, `WriteLocal`, `Binary`, `CallDirect`,
`CallValue`, `LoadGlobal`, `StoreGlobal`, `MakeRecord`, `LoadField`, `StoreField`, `MakeEnum`,
`TestVariant`, `Intrinsic`, `MakeClosure`, and `CellRead`/`CellWrite`. Do not encode an IR instruction
as an untyped opcode plus an arbitrary vector of integers.

Use block parameters for merge values rather than general SSA phi instructions. Mutable FPAS locals
remain explicit locals in IR. This keeps structured lowering straightforward while giving a later
Cranelift frontend a conventional CFG.

## Register bytecode

### Instruction representation

Every in-memory instruction is exactly eight bytes:

```rust
#[repr(transparent)]
pub struct Instruction(u64);
```

The low eight bits are an `Opcode`. The remaining 56 bits use an opcode-declared form. Required forms:

| Form | Payload | Typical use |
|---|---|---|
| `ABC` | three `u16` operands + one `u8` auxiliary field | arithmetic, fields, small calls |
| `ABx` | one `u16` operand + one `u32` operand | constants, globals, jumps |
| `Ax` | one `u48` logical payload exposed through checked accessors | reserved metadata references |

Packing and unpacking must use shifts and masks in safe constructors. Never serialize the native
memory representation, transmute between integers and instructions, or cast byte slices to
instruction slices. Add compile-time and runtime tests for the eight-byte size.

`Opcode` uses `#[repr(u8)]` with explicit discriminants. Unknown discriminants are decoder errors.
Changing an opcode number or operand interpretation requires incrementing `BYTECODE_VERSION`.

### Numeric operands

Use transparent newtypes instead of interchangeable integers:

- `Register(u16)`; `u16::MAX` is reserved as `NO_REGISTER` and never addresses a frame;
- `ConstantId(u32)`;
- `StringId(u32)`;
- `FunctionId(u16)`; the executable rejects more functions than this direct-call representation can
  address;
- `GlobalId(u32)`;
- `RecordTypeId(u16)` and record-local `RecordFieldId(u16)`;
- `EnumTypeId(u16)` plus executable-wide `EnumVariantId(u16)` entries that reference their owner type;
- `IntrinsicId(u16)`;
- `InstructionAddress(u32)` for diagnostic and code ranges.

If an instruction cannot represent a valid program, code generation returns a compiler diagnostic.
Never truncate with `as`. Conversion from collection lengths uses `try_from` and names the exhausted
resource in the error.

### Functions and frames

The executable owns one contiguous instruction vector and a dense function table. Each `FunctionInfo`
contains a half-open code range, arity, result convention, register count, capture count, canonical
name ID, and flags such as `uses_spawn_tasks`.

Function zero is the root initializer/entry function. It returns `Unit`; there is no special root
operand stack. Test bundles and hosted callbacks select a `FunctionId`, never a raw instruction
offset or function name.

A worker owns one `Vec<Value>` register store. Each call appends `register_count` initialized slots and
records a frame base. Bytecode registers are relative to that base. A frame stores:

- caller `FunctionId` and return instruction;
- caller frame base;
- callee frame base and register count;
- destination register for the returned value;
- execution context needed by task/callback policy.

Arguments occupy callee registers `0..arity`. Captures immediately follow parameters. Remaining
registers initialize to `Unit`. Return writes directly into the caller destination and truncates the
callee window.

### Calls

`CallDirect` contains destination register, numeric function ID, contiguous argument-window base, and
argument count. `CallValue` substitutes a function-value register for the direct function ID. The
verifier checks arity, ranges, and that the argument window lies inside the caller frame.

Both use the `ABC` form: `A = destination`, `B = FunctionId` or callee register, `C = argument base`,
and the auxiliary byte is the argument count. The executable therefore supports at most 65,536 table
slots and 255 arguments per call; reserve a lower limit if existing language/compiler limits require
it. Overflow is a compile/link error, never truncation.

The bytecode generator may emit moves into a contiguous scratch window. It must first prefer allocating
expression results directly into that window. Add owned-move bytecode only after profiling proves
reference-count clones remain material; it is not part of the initial cutover.

### Globals and layouts

Globals are dense slots. The runtime uses `RwLock<Vec<Option<Value>>>` initially to preserve shared
task access and undefined-before-initialization diagnostics. Numeric lookup removes hashing and case
folding but does not weaken synchronization.

Record field order is a compile/link-time layout. A record value retains shared layout metadata for
diagnostic formatting and stores only positional values. `LoadField` and `StoreField` use a checked
field slot; no string scan occurs.

Enum values similarly carry numeric type/variant identity plus shared layout metadata for formatting.
Variant tests compare IDs. Associated fields remain positional.

`MakeRecord` and `MakeEnum` use `ABC` with a contiguous value window: destination in `A`, record type
or executable-wide enum variant ID in `B`, and argument base in `C`. The validated layout supplies the
field count, so the encoding does not reduce the current record-field limit. The auxiliary byte is
reserved and must be zero for these opcodes.

First-class functions carry `FunctionId`, captures, task-bound state, and a diagnostic name reference.
Direct and callback invocation dispatch by ID. Names remain metadata only.

## Sparse source maps

Store a sorted vector of source runs:

```text
(instruction_start, source_id, line, column)
```

Emit a run only when the effective source location changes. Function boundaries must start a run even
if they repeat the preceding location. The verifier requires strictly increasing addresses and valid
source IDs.

The dispatch loop tracks only function ID and instruction address. Error creation resolves the closest
preceding run with binary search. Normal instructions do not read or copy `SourceLocation`.

## Static specialization first

FPAS semantic analysis already distinguishes integer, real, boolean, string, and many aggregate
operations. Emit typed opcodes directly. Do not add adaptive counters for operations whose types are
known.

After the base rewrite is measured, compiler peepholes may add benchmark-proven superinstructions such
as compare-and-branch or increment-and-branch. Each superinstruction must:

- preserve the exact error and overflow behavior of its component instructions;
- have explicit verifier rules and malformed-bytecode tests;
- improve at least one registered benchmark without a material suite regression;
- remain optional in code generation so differential tests can compare fused and unfused execution.

Adaptive specialization is reserved for genuinely dynamic generic operations and only after profiles
show those operations dominate. It must quicken an execution-private code copy, never mutate the
portable shared executable used by concurrent VMs.

## Deferred native backend boundary

The future native backend consumes `fpas-ir`, not `.fpascp` bytes and not VM internals. This plan must
not add a `Backend` trait, native ABI types, Cranelift value types, or target configuration merely for
that future.

The only preparation required now is keeping IR free of interpreter-specific stack effects and keeping
hosted operations explicit. When Cranelift is approved later, it receives its own plan and platform
matrix.
