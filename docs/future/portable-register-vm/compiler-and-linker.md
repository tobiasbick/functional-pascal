# Compiler and linker implementation guide

## Compiler contract

The compiler preserves the analyzed program's meaning while changing only its internal representation.
Semantic analysis remains authoritative for types, call targets, properties, events, captures, record
defaults, and standard-library resolution. Lowering must not reimplement name resolution by guessing
from source spellings.

## AST and semantic metadata to IR

### Context ownership

The lowering context owns:

- canonical symbol-to-ID maps populated from semantic results;
- current function, block, lexical scope, and loop targets;
- local declarations and capture bindings;
- source-span interning;
- record/enum layout requests;
- intrinsic references;
- accumulated structured lowering diagnostics.

Do not keep dozens of unrelated maps directly on one compiler struct. Group immutable semantic inputs
in a borrowed `SemanticInputs` structure and mutable lowering state by concern.

### Evaluation order

FPAS evaluation order is observable through calls, I/O, mutations, panics, and tasks. Preserve the
current left-to-right order exactly. A register representation is not permission to reorder operations.

For each expression lower child expressions in current semantic order, then emit the parent operation.
Control-flow short-circuiting remains explicit blocks. Never evaluate both branches merely because
register destinations are available.

### Source spans

Attach the source span of the semantic operation that can fail or be reported. Synthetic register
moves inherit no location unless they are the only instruction representing an observable source
operation. At bytecode selection, source runs change only when the effective span changes.

Diagnostics raised during lowering retain existing diagnostic codes and actionable help text. New
internal limits use a dedicated compile diagnostic naming the exhausted register/table resource.

### Locals and block values

- Parameters are ordered locals and initially occupy entry values.
- Immutable and mutable locals both have stable `LocalId`; mutability is checked by sema, not IR.
- `ReadLocal` yields a typed `ValueId`; `WriteLocal` consumes a value and updates the local state.
- Captured mutable locals lower to cells at the semantically determined capture boundary.
- Branch merge values use block parameters. Do not synthesize hidden FPAS variables.
- Loop `break` and `continue` target explicit blocks and carry any required block arguments.

### Calls

Sema-resolved direct calls lower to `CallDirect(FunctionSymbol, arguments)`. First-class or bound calls
lower to `CallValue(value, arguments)`. Standard-library calls lower to `Intrinsic` only when the
current compiler already treats them as intrinsic behavior.

Do not store a callable name in an ordinary call operation. Names belong to symbolic object metadata
before linking and diagnostic string metadata after linking.

### Records and enums

Use declaration order from semantic types. A record literal lowers values in source-observable
evaluation order, then provides a layout mapping so bytecode construction places them in declaration
slots. Default expressions execute in the same order as current lowering.

Field access receives a symbolic `(record type, field)` reference from sema and becomes a numeric slot
at object/link time. Anonymous record layouts receive deterministic structural keys based on ordered
field names and lowered field types; never use allocation addresses or hash iteration order.

Enum construction and tests carry symbolic type/variant references until linking. Associated data is
positional in declaration order.

### Closures

Capture order comes exclusively from semantic capture metadata. Store capture kind explicitly:
immutable value, mutable cell, or enclosing cell. Nested routine references and anonymous closures use
the same IR closure operation. Do not retain separate runtime mechanisms after cutover.

## IR validation before code generation

Reject at least:

- missing function/block/value/local/type references;
- use before definition inside a block;
- block argument count or type mismatch;
- instruction operand/result type mismatch;
- missing or multiple terminators;
- branch targets outside the function;
- direct call arity/type mismatch;
- invalid closure capture count/order/type;
- return type mismatch;
- illegal fallthrough;
- integer overflow while assigning deterministic IDs.

Compiler bugs return structured internal compiler diagnostics; production code must not panic.

## Register allocation

Use a deterministic linear-scan allocator over reverse-postorder blocks.

1. Number IR operations deterministically.
2. Compute last use for each `ValueId` across blocks.
3. Pin parameter, capture, and language-local registers for the function lifetime in the first
   implementation.
4. Allocate temporary registers from the lowest free register number.
5. Release a temporary after its last use.
6. Reserve contiguous call windows while lowering a call; prefer directing argument expression results
   into the window to avoid moves.
7. Reserve `u16::MAX`; reject a function requiring more addressable registers.
8. Record the high-water mark as `FunctionInfo.register_count`.

The first correct allocator may conservatively pin more values. Any later reuse improvement needs
verifier tests plus benchmark evidence. Do not implement graph coloring.

### Alias-safe destinations

Each opcode declares whether its destination may alias an input. Arithmetic and comparisons may write
an aliased destination after copying scalar operands into Rust locals. Aggregate mutation operations
must preserve FPAS copy-on-write behavior when destination aliases base or value registers.

The code generator inserts a move when an opcode's alias restriction would otherwise be violated. The
verifier checks restrictions that are required for correctness.

## Instruction selection

Instruction selection is a total match over IR operations. Category modules return a concrete checked
`Instruction`; they do not mutate raw bit fields.

Use typed scalar opcodes whenever sema provides a concrete type. Keep dynamic numeric/comparison
opcodes only for erased generic bodies. Preserve current integer wrapping, division/modulo errors,
minimum-integer negation behavior, shift bounds, IEEE-754 representation, string comparison, truthiness,
and panic semantics.

Branch targets are function-relative instruction addresses while compiling and become validated
absolute addresses or `(FunctionId, relative address)` according to the final executable choice. Pick
one representation and use it everywhere. The recommended representation is function-relative `u32`
because it prevents cross-function jumps structurally.

## Relocatable unit objects

Each object contains:

- owner identity and public semantic interface;
- ordered symbolic definitions and imports;
- independently encoded functions with local registers and local control-flow addresses;
- persistent constants and strings;
- symbolic global, function, intrinsic, record, field, enum, and variant references;
- record/enum layout definitions or imports;
- sparse source runs with object-local source IDs;
- compiler, IR, bytecode, source, option, and dependency identity data.

Object code may use object-local table indices plus explicit relocation records. It must never carry a
raw Rust pointer, `usize`, AST node, semantic type allocation, or host path.

## Deterministic linking

The linker receives dependency-first reachable unit objects followed by the root program object.
Determinism rules:

1. Preserve the validated dependency order from the project graph.
2. Within an object, retain declaration/source order where semantics make it observable.
3. Canonical symbol comparisons are ASCII case-insensitive; stored canonical keys are lowercase.
4. Assign function/global/type IDs by object order then canonical symbol order, with the root entry
   fixed as function zero.
5. Assign field and variant IDs by declaration order inside their layout.
6. Merge constants by persistent bit identity, including exact real-number bits and NaN payloads.
7. Build the string table in first-reference order after all preceding orders are deterministic.
8. Never iterate a `HashMap` to produce encoded order; use indexed vectors or sorted maps.

If source declaration order and canonical sorting conflict for an observable initializer, declaration
order wins for execution while numeric table assignment may use a separate deterministic order.

## Link validation

Before relocation:

- validate every object format and identity;
- reject duplicate definitions case-insensitively;
- enforce public visibility at dependency boundaries;
- resolve every import to the required symbol kind;
- confirm callable implementation, signature, and capture metadata;
- unify type layouts and reject same-name incompatible layouts.

During relocation:

- checked-convert every ID and offset;
- ensure each relocation kind matches its opcode operand;
- ensure function-local branches remain in the function;
- merge source files and rebase source IDs;
- preserve root/unit initialization order.

After relocation, run the full executable verifier. Link success is impossible without verifier success.

## Cutover discipline

Do not teach the old stack linker to understand half of the new instruction set. Add the register
object/linker path alongside it under tests, switch all production consumers once complete, then delete
the old path in the immediately following phase.

At the end, `fpas-compiler` must no longer import stack `Op` or emit directly into a mutable `Chunk`.
`fpas-linker` must not resolve function calls or fields by runtime strings.
