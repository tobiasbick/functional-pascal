# Separately Compiled Units: Implementation Plan

Status: planned, no implementation started.

Architecture and product direction: [`libraries.md`](libraries.md).

## Fixed scope

This plan implements the decisions recorded at the top of `libraries.md`:

- Every unit is compiled independently.
- `Example.fpas` produces the source-adjacent sidecar `Example.fpascu`.
- `fpas check`, `fpas run`, and `fpas test` rebuild missing, stale, or incompatible sidecars automatically.
- The first version has no required `fpas build`, `.fpaslib` container, project-local build directory, or global cache.
- The work adds no explicit unit initialization or finalization syntax.
- Existing top-level constant and variable initializers remain startup code and execute in dependency order.
- Sources and manifests remain authoritative; `.fpascu` files are derived and replaceable.
- Unit dependency cycles remain errors.
- There is no backward-compatibility requirement for intermediate artifact formats.

## Definition of success

The feature is complete when:

1. A project unit can be parsed, semantically analyzed, compiled, serialized, loaded, and linked without merging its declarations into the consumer's AST.
2. A valid unchanged `.fpascu` avoids parsing, import rewriting, semantic analysis, and code generation for that unit.
3. A private implementation-only change rebuilds the changed unit and relinks the program without semantically rebuilding consumers whose dependency interface hashes remain unchanged.
4. An exported interface change invalidates direct and transitive consumers.
5. Program, library, workspace, test-project, and source standard-library behavior remains equivalent to a clean source build.
6. `Std.Tui` is built from independently reusable compiled units.
7. The obsolete source-declaration merge path is removed after equivalence coverage passes.

Performance claims must be based on measured cold and warm builds. Correct reuse is verified with deterministic build events or counters, not timing alone.

## Current architecture to replace

The current pipeline is:

```text
.fpasprj / .fpasworkspace
        │
        ▼
fpas-project loads every source and dependency
        │
        ▼
fpas-project resolves reachable units, clones declarations,
renames imported symbols, and merges everything into one Program AST
        │
        ▼
fpas-sema analyzes the complete merged Program
        │
        ▼
fpas-compiler emits one final fpas-bytecode Chunk
        │
        ▼
fpas-vm executes the Chunk
```

Relevant ownership boundaries:

- `crates/fpas-project/src/loading/` — manifests, sources, dependencies, parse cache.
- `crates/fpas-project/src/link/` — reachability, visibility, imports, name rewriting, AST merging.
- `crates/fpas-sema/src/check/` — whole-program symbol registration and semantic analysis.
- `crates/fpas-compiler/src/compiler/` — whole-program bytecode generation.
- `crates/fpas-bytecode/src/chunk.rs` — final bound code, constants, locations, and function offsets.
- `crates/fpas-cli/src/cli_check.rs`, `cli_run.rs`, and `cli_test/` — duplicate orchestration of load, merge, compile, and run.
- `crates/fpas-project/src/standard_library.rs` — parsed source standard-library cache for one process.

The parser already represents a unit as a name, one `uses` list, and declarations. There is no source-level interface/implementation split and no unit body. Public and private declarations must therefore be separated semantically without changing FPAS syntax.

## Target crate and module layout

The intended ownership after implementation is:

```text
crates/
  fpas-unit/                         — NEW: compiled-unit data and .fpascu format
    src/
      lib.rs
      identity.rs                    — source/compiler/options/dependency identities
      interface/
        mod.rs
        types.rs                     — stable exported type representation
        symbols.rs                   — stable exported symbol representation
        hash.rs                      — deterministic interface hashing
      object/
        mod.rs
        code.rs                      — relocatable code and local pools
        imports.rs                   — required external definitions
        exports.rs                   — provided definitions
        relocation.rs                — constant/code/function relocations
      format/
        mod.rs
        header.rs                    — magic and format version
        read.rs                      — bounded, diagnostic reader
        write.rs                     — deterministic writer
      sidecar/
        mod.rs
        path.rs                      — Foo.fpas -> Foo.fpascu
        atomic.rs                    — same-directory temporary write + replace
        validation.rs                — reuse/rebuild decision

  fpas-project/
    src/
      unit_graph/                    — NEW: source/project graph without AST merging
        mod.rs
        model.rs                     — unit path, name, origin, direct uses
        resolve.rs                   — reachability and project export policy
        order.rs                     — stable topological order and cycles
      link/                          — TEMPORARY: existing source-merging linker

  fpas-sema/
    src/
      unit/                          — NEW: independently analyzable unit boundary
        mod.rs
        imports.rs                   — install dependency interfaces
        interface.rs                 — export public declarations
        analyze.rs                   — check one unit implementation

  fpas-compiler/
    src/
      unit/                          — NEW: relocatable unit code generation
        mod.rs
        declarations.rs
        startup.rs                   — top-level const/var initializer code
        relocations.rs
      program_object.rs              — NEW: relocatable main-program output

  fpas-linker/                       — NEW: compiled objects -> final Chunk
    src/
      lib.rs
      layout.rs                      — final offsets and pool indices
      constants.rs                   — deterministic pool merge/remapping
      code.rs                        — opcode relocation
      functions.rs                   — function table merge
      startup.rs                     — dependency-ordered startup sections
      source_map.rs                  — final source table and locations
      diagnostics.rs

  fpas-build/                        — NEW: reusable build orchestration
    src/
      lib.rs
      graph.rs                       — build nodes and dependency fingerprints
      invalidation.rs                — rebuild/relink/reuse classification
      schedule.rs                    — dependency-ordered compilation
      compile.rs                     — Sema/compiler coordination
      sidecars.rs                    — load, validate, and atomically publish
      standard_library.rs
      report.rs                      — deterministic events/counters for tests

  fpas-cli/
    src/
      build_pipeline/                — NEW: thin CLI adaptation over fpas-build
        mod.rs
        check.rs
        run.rs
        test.rs
```

`fpas-cli` must not become the owner of compiled-unit semantics or invalidation. `fpas-build` provides one pipeline shared by check, run, and test.

## Cross-cutting invariants

Every implementation stage must preserve these rules:

- Canonical unit and symbol names remain case-insensitive.
- Unit source paths stay available for diagnostics but absolute machine paths do not affect interface hashes or deterministic artifact bytes.
- `[exports].units` remains a project-boundary rule; unit `private` remains a declaration-boundary rule.
- Only reachable units are linked into programs. Checking a library still validates and builds all its units.
- Sidecars are never accepted solely because their timestamp is newer than the source.
- Invalid or corrupt sidecars never cause a compiler panic.
- A sidecar is written only after the complete object validates.
- Concurrent commands must not observe partially written sidecars.
- `fpas test` builds shared unit dependencies before starting test workers instead of allowing workers to race on the same sidecar.
- Plain single-file programs without project units continue to compile without creating an artifact.
- `.fpascu` files are not treated as sources by manifest globs, directory checks, formatting, or test discovery.

## Phase 1 — Extract a reusable unit graph

Status: [ ]

### Objective

Separate project/unit graph construction from AST declaration merging while leaving current behavior unchanged.

### Work

- Add `fpas-project::unit_graph` with public, documented graph types.
- Record for every unit:
  - canonical and display name;
  - source path;
  - owning project manifest;
  - source origin and export policy;
  - direct `uses`;
  - stable source ID for diagnostics.
- Move or reuse reachability, cycle detection, stable topological sorting, duplicate-unit detection, and project visibility checks from `link/`.
- Make the existing source-merging linker consume the new graph so there is one graph implementation.
- Keep import ambiguity and symbol rewriting in the existing linker during this phase.
- Split `loading/own.rs` before adding responsibilities if it crosses the project file-size guideline.

### Tests

- Positive: path and workspace libraries produce the expected graph and stable order.
- Negative: missing units, duplicate names, dependency cycles, and imports of non-exported units.
- Edge: differently cased unit names, diamond dependencies, unused library units, and multiple projects referencing one dependency.
- Regression: existing project/linker and CLI project tests remain unchanged and pass.

### Exit criteria

- The existing compiler output is unchanged.
- No second implementation of reachability or export policy exists.
- `fpas-project` can return a resolved graph without producing a merged `Program`.

## Phase 2 — Define `.fpascu` identity, format, and safe sidecar I/O

Status: [ ]

### Objective

Create the versioned compiled-unit envelope before storing real semantic or bytecode payloads.

### Work

- Add the `fpas-unit` crate and workspace membership.
- Define a fixed magic header, explicit format version, bounded field lengths, and deterministic field order.
- Add stable digests for:
  - source content;
  - semantic interface;
  - semantic compiler options;
  - compiler/bytecode compatibility;
  - ordered direct dependency interface hashes.
- Derive sidecar paths with `Path::with_extension("fpascu")`.
- Store source identity without making absolute paths part of deterministic content.
- Implement read outcomes that distinguish:
  - reusable;
  - missing;
  - stale;
  - incompatible format/compiler;
  - corrupt;
  - I/O failure.
- Treat missing, stale, incompatible, and corrupt sidecars as rebuild candidates when the source directory is writable.
- Write a uniquely named temporary file in the source directory, flush it, validate it, and atomically replace the destination.
- Add `*.fpascu` to the repository `.gitignore`.
- Do not silently redirect sidecars to another directory. A read-only directory accepts a valid sidecar or reports an actionable build error.

### Tests

- Exact source-to-sidecar path mapping on Windows and Unix-style paths.
- Deterministic bytes for identical logical objects.
- Round trips for minimum and maximum supported field shapes.
- Rejection of bad magic, unsupported version, truncation, invalid lengths, invalid enum tags, and trailing corruption.
- Atomic replacement preserves the previous valid object if writing fails.
- Parallel writers never expose a partial object.
- Changed source, compiler identity, option fingerprint, or dependency interface hash is classified correctly.

### Exit criteria

- The empty/skeleton `.fpascu` format can be written and read safely.
- Format parsing is fuzzable and contains no unchecked allocation based on file data.
- No production compilation path writes sidecars yet.

## Phase 3 — Introduce stable semantic unit interfaces

Status: [ ]

### Objective

Represent everything a consumer needs to type-check without loading the dependency implementation AST.

### Work

- Define serialization-oriented interface types in `fpas-unit::interface`; do not serialize `fpas-sema::Ty` or scope internals directly.
- Cover public:
  - constants and values needed at compile time;
  - variables and mutability;
  - functions, procedures, parameters, generic parameters, and constraints;
  - records, fields, methods, static routines, properties, events, and defaults needed by consumers;
  - enums, variants, associated data, and backing values;
  - aliases, arrays, dictionaries, options, results, tasks, and recursive named references.
- Define canonical ordering and spelling rules before hashing.
- Add conversion between interface types and Sema's internal `Ty`/`Symbol`.
- Add a unit-interface extraction API which exports public declarations and excludes private declarations.
- Install imported interfaces into a unit/program root scope with the same short-name and qualified-name behavior as the current linker.
- Preserve lazy ambiguity diagnostics: importing two equal short names is allowed until the short name is used.
- Include enough owner information for record methods, properties, events, and fully qualified calls.

### Tests

- Round-trip every interface type and symbol category.
- Public declarations appear; private declarations do not.
- Interface hashes ignore private routine bodies and source spans.
- Interface hashes change for every observable exported signature/layout/value change.
- Declaration order that is semantically equivalent has the documented deterministic behavior.
- Recursive records/enums and generic callable types round-trip without deep cloning regressions.
- Imported short and fully qualified names reproduce existing visibility and ambiguity behavior.

### Exit criteria

- A consumer Sema context can be populated from dependency interfaces alone.
- Interface extraction and import have equivalence tests against symbol maps produced by the current merged-AST path.

## Phase 4 — Analyze one unit independently

Status: [ ]

### Objective

Replace whole-program semantic analysis of library implementations with dependency-interface-driven unit analysis.

### Work

- Add an Sema entry point accepting:
  - one parsed `Unit`;
  - resolved dependency interfaces;
  - owning-unit identity;
  - project visibility context.
- Split declaration registration from routine-body checking where necessary.
- Produce:
  - diagnostics;
  - the exported semantic interface;
  - all compiler metadata currently returned by `analyze_with_types`;
  - private implementation symbol information required only for this unit.
- Qualify unit-owned names without first rewriting the complete AST into a program.
- Keep existing source spans and source IDs intact.
- Analyze units in topological order.
- Add an equivalent program-analysis entry point that consumes compiled unit interfaces rather than their declarations.
- Retain the merged-AST Sema path only as an internal comparison oracle during migration.

### Tests

- Positive unit tests for every declaration/type/callable category.
- Negative tests for private access, missing imports, ambiguous short names, wrong signatures, and invalid exported types.
- Edge tests for nested routines, closures, record defaults, properties, events, enum data, generic constraints, and transitive qualified references.
- Equivalence tests compare diagnostics and inferred compiler metadata between merged and independent analysis.

### Exit criteria

- Every existing project semantic test passes through the independent-unit API in dedicated equivalence tests.
- A valid dependency implementation AST is no longer required to analyze a consumer.

## Phase 5 — Emit relocatable unit and program objects

Status: [ ]

### Objective

Compile independently analyzed units without assigning final whole-program addresses or pool indices.

### Work

- Define relocatable object code in `fpas-unit::object`.
- Split the compiler's final-`Chunk` assumptions from declaration and statement lowering.
- Add `compile_unit` and `compile_program_object` APIs.
- Give each object local:
  - code offsets;
  - constants;
  - function entries;
  - globals;
  - source locations.
- Record external imports and exported definitions explicitly.
- Record relocation sites for every opcode operand that refers to:
  - a constant-pool entry;
  - a code address;
  - a global;
  - a named function or closure target.
- Centralize operand relocation so a new opcode cannot silently omit link handling.
- Put top-level constant and variable initializer instructions in a distinct startup section.
- Do not add explicit unit initialization/finalization syntax.
- Serialize the semantic interface and compiled implementation into `.fpascu`.

### Tests

- Unit objects for constants, globals, calls, closures, nested routines, jumps, records, enums, properties, events, and intrinsics.
- Exhaustive relocation coverage for every `Op` variant with operands.
- Local code jumps remain local and are correctly rebased.
- Interface-only consumers do not load implementation ASTs.
- Serialization round trips retain bytecode invariants and source locations.
- Oversized pools, functions, code addresses, and relocation indices produce diagnostics instead of truncation.

### Exit criteria

- A unit and a main program can be compiled to relocatable objects.
- No unit object assumes it starts at final code offset zero in the executable image.

## Phase 6 — Link objects into the final `Chunk`

Status: [ ]

### Objective

Build the VM's existing executable `Chunk` from a program object and reachable unit objects.

### Work

- Add the `fpas-linker` crate.
- Resolve imports against canonical exported definitions.
- Reject missing definitions, duplicate definitions, signature mismatches, and unexpected private symbols.
- Compute deterministic layout in dependency/topological order.
- Merge and deduplicate constants while remapping every constant operand.
- Rebase jumps, function entries, and source-location streams.
- Assign globals and callable names consistently with current VM lookup.
- Concatenate top-level declaration startup sections dependency-first, then execute the program body.
- Emit exactly one final `Halt`.
- Validate the final `Chunk` with existing and expanded invariants.

### Tests

- Positive linking of one unit, chains, diamonds, multiple libraries, and mixed intrinsic/source standard units.
- Negative unresolved, duplicate, private, incompatible, corrupt, and overflow cases.
- Edge cases for empty units, units containing only types, recursive call graphs, closures, globals, and identical constants from multiple units.
- Byte-for-byte deterministic final chunks for identical ordered inputs where practical.
- Behavioral equivalence in `fpas-vm` between source-merged and object-linked chunks.

### Exit criteria

- Object-linked programs run in the existing VM without VM compatibility branches.
- The final `Chunk` format remains focused on execution rather than artifact persistence.

## Phase 7 — Build graph, invalidation, and automatic sidecars

Status: [ ]

### Objective

Reuse valid sidecars and rebuild the minimum dependency closure.

### Work

- Add the `fpas-build` crate.
- Classify each unit as:
  - reuse;
  - rebuild implementation;
  - rebuild interface and implementation;
  - relink only;
  - error because rebuilding is impossible.
- Build dependencies before consumers and propagate new interface hashes.
- Distinguish source-hash changes from interface-hash changes.
- Reuse a consumer when all recorded direct dependency interface hashes still match.
- Build all units for `kind = "library"` checks; build only reachable units for programs and test entries.
- Publish sidecars only after Sema, codegen, serialization, and self-validation succeed.
- Coordinate one invocation so each source unit is compiled at most once.
- Produce structured build events and counters for deterministic tests:
  - parsed;
  - interface analyzed;
  - implementation analyzed;
  - compiled;
  - sidecar reused;
  - relinked.
- Keep event reporting internal or diagnostic-level; do not make unstable text part of the user CLI contract.

### Tests

- Cold build creates all required sidecars.
- Warm build reuses all unchanged sidecars.
- Private body change rebuilds one object and relinks without rebuilding consumer Sema.
- Exported change rebuilds direct and transitive consumers.
- Unreachable source changes do not affect a program build.
- Failed compilation leaves the last valid sidecar untouched but does not reuse it for changed source.
- Concurrent top-level commands leave valid complete sidecars.
- Read-only source trees work with valid sidecars and fail clearly when rebuilding is required.

### Exit criteria

- Reuse and invalidation are proven through events/counters.
- The pipeline never depends on timestamp ordering for correctness.

## Phase 8 — Integrate check, run, workspaces, and tests

Status: [ ]

### Objective

Make compiled units the normal behavior of all project-aware CLI commands.

### Work

- Route project and workspace `check` through `fpas-build`.
- Route project `run` through object compilation and linking.
- Preserve plain single-file `run` and `check` behavior when no project units are involved.
- For workspace check, reuse sidecars across members and compile shared libraries once.
- For test projects:
  - build helper/library units once before worker startup;
  - reuse unit objects across individual and shared-image test paths;
  - keep setup/teardown and sidecar override semantics unchanged;
  - prevent worker-side sidecar races.
- Consolidate repeated build logic currently in `cli_check.rs`, `cli_run.rs`, and `cli_test`.
- Keep automatic builds silent on successful reuse unless an existing verbosity mechanism requests details.
- Do not require an explicit `fpas build`.

### Tests

- Existing CLI project, workspace, visibility, standard-library, and test-runner suites.
- Cold/warm check and run behavior.
- Workspace members sharing a library.
- Test jobs `1`, `0`, and greater than one with the same shared sidecars.
- Directory and single-file commands do not create unexpected sidecars.
- `.fpascu` files are ignored by discovery, formatting, and test collection.

### Exit criteria

- `check`, `run`, and `test` all use one build pipeline.
- User-visible behavior is documented under `docs/pascal/program-structure/`.

## Phase 9 — Precompile the source standard library and validate `Std.Tui`

Status: [ ]

### Objective

Apply the general compiled-unit pipeline to bundled and overridden source standard libraries.

### Work

- Replace the one-process parsed standard-library cache with normal `.fpascu` validation and reuse.
- Generate compatible standard-library sidecars during distribution staging and place them beside copied `.fpas` sources.
- Do not commit generated sidecars to the repository.
- Keep `--std-lib` authoritative:
  - use matching sidecars beside the selected sources;
  - automatically rebuild missing/stale sidecars when writable;
  - produce an actionable error when read-only sources lack compatible sidecars.
- Compile the `Std.Tui` facade and every reachable internal unit independently.
- Ensure intrinsic standard units continue to use their existing registration/lowering path.
- Add cold/warm build measurements that separate:
  - loading and validation;
  - Sema;
  - code generation;
  - linking;
  - VM runtime for tests.

### Tests

- Bundled library with matching sidecars.
- Source override with matching, missing, stale, corrupt, and compiler-incompatible sidecars.
- Source/intrinsic unit collision rules.
- `Std.Tui` check, representative run, headless tests, and full regression suite.
- A private internal `Std.Tui` implementation change does not semantically rebuild unaffected consumers.
- An exported `Std.Tui` interface change invalidates the expected dependency closure.

### Exit criteria

- Warm `Std.Tui` builds demonstrably reuse internal unit objects.
- Behavior and diagnostics match clean source compilation.
- Performance results are recorded in the completed plan or a linked benchmark note.

## Phase 10 — Remove the source-merging production path

Status: [ ]

### Objective

Finish the migration without permanent compatibility layers.

### Work

- Make compiled-unit interfaces and objects the only project/library production path.
- Remove declaration cloning and whole-program source merge code that is no longer required.
- Remove comparison-only modes and duplicate orchestration.
- Retain only source graph, visibility, and import rules still owned by `fpas-project`.
- Delete obsolete parsed-unit caches superseded by sidecars.
- Reassess module sizes and split any files approaching the project limit.
- Update crate descriptions and public Rust documentation.

### Tests and verification

- `cargo fmt`
- `cargo build`
- `cargo test --workspace`
- `fpas test tests/` or `fpas test tests/suite.fpasprj`
- `fpas fmt --check` for any changed FPAS sources
- Clean cold build followed by warm build
- `Std.Tui` targeted checks and headless tests
- `git diff --check`

### Documentation

Move implemented behavior from the future documents into:

- `docs/pascal/program-structure/units.md`
- `docs/pascal/program-structure/projects.md`
- `docs/pascal/program-structure/workspaces.md`
- `docs/pascal/program-structure/cli.md`
- `docs/pascal/std/testing/test.md`

After the implementation and documentation are complete, remove the completed future-plan documents and their index entries rather than retaining stale roadmap text.

### Exit criteria

- No production project build merges dependency declarations into a consumer `Program`.
- No obsolete compatibility alias, format reader, or source-merge fallback remains.
- All acceptance criteria in `libraries.md` and this plan pass.

## Required diagnostic coverage

The implementation is incomplete unless diagnostics cover:

- malformed or unsupported `.fpascu`;
- stale source hash;
- stale dependency interface hash;
- compiler/bytecode incompatibility;
- read-only source directory requiring a rebuild;
- missing imported unit or symbol;
- private unit or symbol access;
- duplicate unit or exported symbol;
- dependency cycle with the full unit path;
- relocation and pool/address overflow;
- atomic-write failure with both source and sidecar paths;
- inability to replace a sidecar held open by another process.

Every diagnostic should state whether the compiler will rebuild automatically or which file/directory condition the user must correct.

## Required regression matrix

Each relevant phase must add positive, negative, and edge coverage across:

| Area | Required coverage |
| --- | --- |
| Project kinds | program, library, test |
| Dependency form | relative path, absolute path, workspace name, transitive |
| Visibility | exported unit, internal unit, public symbol, private symbol |
| Graph shape | chain, diamond, unreachable unit, duplicate, cycle |
| Source state | cold, warm, private change, public change, corrupt sidecar |
| Filesystem | writable, read-only with valid object, read-only requiring rebuild |
| Commands | check, run, test, workspace discovery, single-file |
| Standard library | bundled, `--std-lib`, intrinsic/source mix, `Std.Tui` |
| Concurrency | one job, automatic jobs, parallel test workers, concurrent commands |
| Language features | records, enums, aliases, generics, closures, properties, events, globals |

## Plan maintenance

- Complete phases in order unless a phase explicitly records a safe dependency-independent extraction.
- Mark a phase complete only after its exit criteria and verification pass.
- Record material architecture deviations in this document before implementing dependent phases.
- Keep implementation details in this plan until they exist; update `docs/pascal/` only when behavior becomes available.
- Do not add GitHub Actions or other repository automation.
