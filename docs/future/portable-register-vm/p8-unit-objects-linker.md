# P8 unit objects and linker implementation

P8 is complete on the inactive register-development path. Production `fpas build`, `fpas run`,
`.fpascp`, and native bundles remain on the stack image until the P9 artifact and CLI cutover. This
phase changes no FPAS syntax, semantics, language documentation, or user-facing backend selection.

## Object contract

`fpas-unit::object::RelocatableObject` is the P8 persistent register object. It contains only
pointer-width-neutral values: fixed-width integers, booleans, UTF-8 strings, ordered vectors, packed
logical instruction words, and explicit symbolic references. It does not contain `usize`, Rust
pointers, AST nodes, semantic type allocations, target triples, host paths, or native-endian data.

Each object owns:

- a canonical lowercase owner and object schema version;
- either an optional root entry or an optional unit initializer, never both; dependency objects
  forbid entries and initializers must use a zero-parameter, zero-capture, Unit-return ABI;
- independently encoded functions with local registers, local branch targets, ABI metadata, task
  flags, and function-local sparse source runs;
- semantic-bit-identity constants, dense globals, record layouts, enum layouts, and source paths;
- canonically ordered definitions and imports with callable, mutability, record, or enum shapes;
- exactly one relocation record for every relocatable instruction operand.

`RelocatableObject::from_executable` converts a verified register executable into that form. Branches
become function-local; constants, calls, closures, globals, records, fields, enum variants, enum
fields, code ranges, strings, and source IDs are either explicitly relocated or deterministically
rebuilt by the linker. Register operands are copied unchanged and are never link relocations.

The bounded object payload codec validates before encoding and after decoding. Vector order is the
wire order; definitions, imports, and relocations must be strictly canonical. Equal inputs therefore
produce equal bytes without relying on `HashMap` iteration. The `.fpascu` envelope version is 3;
versions 1 and 2 are incompatible derived artifacts and are rebuilt, not migrated.

## Link contract

`fpas-linker::link_register_objects` accepts dependency-first unit objects followed by one root
object and returns only `VerifiedExecutable`. It performs these stages in order:

1. validate every object locally, reject unit entries, and validate initializer ABI;
2. collect definitions in a sorted map, reject case-insensitive duplicates, resolve imports, enforce
   public visibility, and compare callable ABI, global mutability, and ordered type layouts;
3. reserve function zero for the root entry, then assign function, global, record, and enum IDs by
   object order plus canonical symbol order;
4. assign record fields and enum variants in declaration order;
5. merge constants by exact persistent identity, preserving signed zero and NaN payload bits;
6. intern strings in deterministic first-reference order;
7. prefix the root with dependency-order initializer calls, concatenate functions, rebase local
   branches/code ranges/source runs, and rewrite every symbolic table operand with checked
   fixed-width conversions;
8. run the complete register executable verifier and return no executable on failure.

Ordinary calls, globals, records, and enum operations leave the linker with numeric IDs only. The
runtime does not resolve those operations through strings.

## Temporary production boundary

The former stack `RelocatableObject` is named `ChunkObject` and is isolated in
`fpas-unit/object/chunk.rs`; its linker remains the production adapter solely until P9 switches
`.fpascp` and CLI consumers. New register APIs do not accept `ChunkObject`, and the register linker
does not import stack `Op`. P10 removes this adapter after the production cutover.

## Incremental build path

`fpas-build` exposes the inactive `build_register_library_units`, `check_register_library_units`,
`build_register_program`, and `check_register_program` APIs. They use the same dependency identity,
interface hashing, source snapshot, atomic publication, build events, and minimum-rebuild engine as
the production stack backend. Backend adapters differ only in object compilation, object codec,
typed sidecar loading, and stack-only numeric source-ID normalization.

Register `.fpascu` payloads use the existing source-adjacent envelope and the P8 register object
codec. `load_register_sidecar` validates envelope identity, interface ownership, object ownership,
object invariants, and hashes before reuse. Missing, stale, old-envelope, corrupt-payload, compiler,
bytecode, option, and dependency mismatches are derived-artifact rebuild conditions. The inactive
register and production stack APIs may replace one another's sidecars; a payload of the other backend
is classified as corrupt and rebuilt. P9 removes that temporary dual-backend condition.

## Test evidence

`fpas-unit/tests/register_object.rs` covers positive conversion of every register table family,
deterministic encoding/round trip, every truncated prefix, object versions, relocation coverage, and
canonical definitions.

`fpas-linker/tests/register_link.rs` covers dependency objects linked into a verified executable and
executed by `RegisterVm`; root function zero; canonical function order; exact constant merging;
source rebasing; imported functions, globals, records, and enum variants; private, missing,
wrong-kind, wrong-arity, incompatible-layout, duplicate-definition, unit-entry, and function-ID
overflow failures; and repeated-link equality.

`fpas-compiler::compile_register_object`, `compile_register_unit_object_with_support`, and
`compile_register_program_object_with_support` prove source-to-IR-to-bytecode-to-object compilation,
symbolic function/global/record/enum imports, transitive independently compiled units, dependency
initialization, final linking, and register-VM execution.

`fpas-build/tests/incremental.rs` exercises a program with transitive units through cold and warm
register builds, asserts byte-identical object payloads and executables, and proves automatic minimum
rebuild for missing, old-envelope, corrupt-payload, and incompatible sidecars. A separate workspace
fixture resolves a program member's `[dependencies].workspace` library and runs its linked units in
`RegisterVm`. Existing production compiler, project dependency, library, workspace, incremental,
and runtime suites remain regression gates while public CLI artifact selection stays unchanged until
P9.

The final P8 verification passed from the repository root:

- `cargo fmt --all -- --check`;
- `cargo build --workspace --locked`;
- `cargo test --workspace --locked`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `fpas fmt --check --list examples tests apps`;
- `fpas test tests/` (`385` passed, `1` intentionally skipped, `0` failed).

## Documentation classification

This is an internal compiler/linker and derived-artifact change. `docs/pascal/` remains unchanged:
accepted FPAS programs and current CLI behavior did not change. Durable implementation evidence stays
under `docs/future/` until the P9/P10 cutover is complete.
