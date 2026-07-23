# Future: Libraries

## Fixed decisions for the implementation plan

These decisions define the scope of the separately compiled unit work:

- **Automatic builds:** `fpas check`, `fpas run`, and `fpas test` automatically build missing, stale, or incompatible compiled units. An explicit `fpas build` may later provide eager compilation, but is not required for normal path or workspace dependencies.
- **Source-adjacent unit objects:** compiling `Example.fpas` writes `Example.fpascu` beside it. `.fpascu` means Functional Pascal compiled unit. These derived files are excluded from source discovery and should be ignored by version control.
- **No new unit lifecycle syntax:** the work does not add explicit unit initialization or finalization blocks. Existing top-level constant and variable initializers remain part of the compiled unit startup code and retain dependency-correct execution order.
- **No initial container or global cache:** the first implementation uses one `.fpascu` sidecar per unit. A `.fpaslib` container and a shared machine-wide cache remain possible later optimizations.
- **Sources remain authoritative:** `.fpasprj` paths, workspace names, `.fpas` sources, and current visibility rules remain the dependency model. A stale or invalid `.fpascu` is rebuilt from its source rather than treated as an independent package.

## Implemented

Source-level library projects:

- `kind = "library"` in `.fpasprj` (units only, no `main`).
- Consumption via `[dependencies].projects` and `[dependencies].workspace`.
- Transitive dependencies, cycle detection, `fpas check` on libraries and workspaces.
- **`[exports].units`** — optional project-level public unit list for dependents (internal units stay library-private).

Spec: [`docs/pascal/program-structure/projects.md`](../pascal/program-structure/projects.md). Examples: [`examples/pascal/library-deps/`](../../examples/pascal/library-deps/), [`examples/pascal/monorepo/`](../../examples/pascal/monorepo/).

## Agreed direction: separately compiled units

Libraries should eventually follow a Turbo Pascal-style compilation model:

1. Compile every unit separately.
2. Store its public semantic interface and compiled implementation.
3. Recompile a unit only when its source or a consumed dependency interface changes.
4. Link the artifacts of units reachable from the program's `uses` graph into the final executable bytecode image.

The current source-level dependency model remains the source of truth. Project dependencies continue to use paths to `.fpasprj` files or workspace member names. Compiled artifacts are derived build outputs, not a replacement dependency syntax.

The final implementation plan should preserve the existing distinction between:

- **Project dependency resolution** — finds library projects, validates dependency cycles and enforces `[exports].units`.
- **Unit compilation** — produces one independently reusable compiled object per unit.
- **Program linking** — selects reachable unit objects and combines them with the program into one executable bytecode image.

## Goals

- Avoid parsing, import rewriting, semantic analysis and bytecode generation for unchanged library units on every consumer build.
- Make unit interfaces the incremental compilation boundary.
- Include only units reachable through `uses` in the final program.
- Keep diagnostics attributable to the original source file and source location.
- Preserve private unit members and project-level `[exports].units`.
- Support source-defined standard-library units with the same compiled-unit model.
- Detect stale or incompatible artifacts deterministically and rebuild them from source.
- Keep the artifact and linker design usable for normal libraries rather than creating a `Std.Tui`-specific mechanism.

## Non-goals

- Package registries or remote package discovery.
- Lockfiles or semver dependency resolution.
- Installing libraries into a global package store.
- Explicit unit initialization or finalization syntax.
- Maintaining compatibility with an obsolete compiled-unit format.
- Shipping source-less third-party libraries as an initial requirement.
- A project-level `.fpaslib` container or shared machine-wide artifact cache in the first implementation.
- Per-symbol dead-code elimination inside an included unit. The first implementation may link the complete implementation of every reachable unit.

## Proposed artifact model

The compiler should produce one **compiled unit object** beside each unit source:

```text
src/
  Geometry.fpas
  Geometry.fpascu
```

The `.fpascu` serialization representation remains an implementation decision. A later project-level artifact such as `.fpaslib` may be a container or index over these unit objects, but it must not collapse the library into one indivisible bytecode chunk.

The logical content of a compiled unit object should include:

### Identity and validation

- Canonical unit name.
- Source content hash.
- Compiler version.
- Bytecode and compiled-unit format version.
- Target/runtime compatibility information when required.
- Ordered direct unit dependencies with the semantic interface hash expected from each dependency.
- Compilation options that affect semantics or emitted bytecode.

### Semantic interface

- Exported types and their complete layouts.
- Exported constants and their values.
- Exported variables, routines, methods, properties and events.
- Routine signatures, calling conventions and generic information required by consumers.
- Visibility information needed to enforce unit `private` and project `[exports].units`.
- A stable semantic interface hash.

Private implementation details should not contribute to the interface hash unless they alter an exported type, value or signature. This allows an implementation-only change to avoid recompiling consumer units.

### Compiled implementation

- Relocatable bytecode for the unit's routines and top-level declaration initializers.
- Constant pool entries owned by the unit.
- Exported symbol definitions.
- Imported symbol references.
- Relocation records for constants, globals, routines, types and bytecode addresses.
- Function entry metadata.
- Startup-code metadata for existing top-level declaration initializers.
- Source maps and any diagnostic metadata required at runtime.

The current executable `Chunk` is a fully bound program image and is not itself a compiled unit object. The linker must translate multiple compiled unit objects into the final `Chunk`, remapping constant indices, globals, functions and instruction addresses as needed.

## Compilation model

Unit compilation should have explicit interface and implementation phases.

### Interface phase

1. Parse the unit.
2. Resolve and load interfaces of units listed in `uses`.
3. Validate the unit's public declarations.
4. Produce the semantic interface and its stable hash.

An implementation plan must define how mutually dependent interfaces are handled. If cyclic unit dependencies remain forbidden, the compiler should report the dependency cycle with the participating unit names and `uses` edges.

### Implementation phase

1. Analyze private declarations and routine bodies against the completed dependency interfaces.
2. Emit relocatable unit bytecode.
3. Record imports, exports, relocations, top-level declaration initializers and source mapping.
4. Atomically store or update the source-adjacent `.fpascu` object.

The build graph should process units in dependency order and may compile independent graph branches in parallel.

## Linking model

The linker should:

1. Start from the program's `uses` list.
2. Resolve the transitive reachable-unit graph.
3. Reject missing, stale or incompatible unit objects before producing a partial image.
4. Enforce library `[exports].units` at project boundaries.
5. Assign final global, constant and function locations.
6. Resolve every imported symbol against exactly one exported definition.
7. Merge unit bytecode and source maps into the final `Chunk`.
8. Emit existing top-level declaration initializer code in deterministic dependency order.
9. Emit the program body after required declaration initializers.

## Incremental invalidation

A unit must be rebuilt when any of these inputs changes:

- Its source content.
- A semantic compilation option.
- The compiler, bytecode format or compiled-unit format becomes incompatible.
- The semantic interface hash of a directly consumed unit changes.
- Project visibility metadata changes in a way that affects its imports or exports.

An implementation-only dependency change with an unchanged semantic interface should require relinking, but should not force semantic recompilation of consumers.

Artifact validation must use recorded hashes and versions. File timestamps alone are not sufficient.

The implementation plan must define:

- Exact `.fpascu` sidecar naming for unusual file names and case-insensitive filesystems.
- Atomic artifact writes.
- Coordination of concurrent attempts to rebuild the same sidecar.
- Cleanup of stale or orphaned `.fpascu` files.
- Behavior after an interrupted build.
- Diagnostics when a source directory is read-only.
- Version-control ignore rules and a future `fpas clean` workflow.

## Standard library and `Std.Tui`

Source-defined standard-library units should use the same interface, object and linker formats as project libraries. The compiler distribution may include precompiled objects matching its bundled standard-library sources.

`Std.Tui` is an important acceptance workload because its public facade reaches a large graph of focused internal units. With compiled units:

- Unchanged internal TUI units should not be reparsed or semantically reanalyzed for each consumer.
- Changing one internal implementation should rebuild that unit, relink reachable objects and rebuild dependent units only when its semantic interface changes.
- A program importing `Std.Tui` should receive the same observable behavior and diagnostics as a source-only build.

`--std-lib <directory>` remains a complete source override. Precompiled bundled objects must not be reused when their recorded source, interface or compiler compatibility data does not match the selected standard-library directory. The compiler should automatically rebuild matching `.fpascu` sidecars beside the selected sources. If that directory is read-only and no valid sidecar exists, the diagnostic must name the source and expected sidecar and explain that the directory must contain compatible precompiled objects or be writable.

## CLI and build lifecycle

The implementation plan must provide:

- Automatic sidecar validation and rebuilding for `fpas check`, `fpas run`, and `fpas test`.
- No required library manifest schema change for locating sidecars.
- How `fpas check` validates a `kind = "library"` project without producing a runnable program.
- How `fpas test` shares compiled unit objects across test programs and workers.
- Safe behavior for valid sidecars in a read-only source tree.
- A later decision on whether `fpas build` and `fpas clean` should be added as convenience commands.

These decisions must not introduce install steps merely to consume a path or workspace library during normal development.

## Diagnostics and reproducibility

Failures involving compiled units should name:

- The consuming unit or program.
- The dependency unit and project.
- The artifact path when relevant.
- The expected and actual interface or format version.
- A concrete rebuild or cleanup action.

Given identical sources, compiler version and semantic options, unit interfaces and emitted unit objects should be deterministic. Source-only and artifact-backed builds must produce equivalent executable behavior.

## Implementation-plan inputs

A future implementation plan should break the work into independently verifiable stages covering at least:

1. Stable semantic interface representation and hashing.
2. Unit dependency graph and incremental invalidation.
3. Relocatable bytecode object representation.
4. Serialization and compatibility validation.
5. Unit compiler interface/implementation phases.
6. Linker construction of a final `Chunk`.
7. Source-adjacent sidecar lifecycle and concurrent writes.
8. CLI integration for projects, workspaces, tests and `--std-lib`.
9. Bundled standard-library artifacts.
10. Migration of `Std.Tui` as the primary large acceptance workload.
11. Documentation, diagnostics and positive, negative and edge-case regression coverage.

Each stage should retain a source-only comparison path until artifact-backed output has equivalence coverage. The completed implementation should remove obsolete source-merging paths rather than keeping permanent compatibility layers.

## Acceptance criteria for the eventual feature

- Two programs consuming the same unchanged library reuse its compiled unit objects.
- Editing a private routine body does not semantically recompile consumers when the unit interface hash is unchanged.
- Editing an exported signature invalidates direct and transitive consumers.
- Only units reachable through `uses` are linked.
- Private units and private symbols remain inaccessible across their existing boundaries.
- Dependency cycles, unresolved imports, duplicate exports and incompatible artifacts produce actionable diagnostics.
- Existing top-level declaration initializer order is deterministic and dependency-correct.
- Source locations in compile-time and runtime diagnostics still identify the original `.fpas` file.
- Artifact-backed and clean source-only builds pass equivalent behavior tests.
- `Std.Tui` consumers reuse unchanged internal compiled units.
- `fpas test` safely reuses unit objects across compatible test builds.

## Further library work

- Finer-grained export control (per-symbol export tables on the project, re-export lists, etc.) beyond unit-level `[exports]` and per-unit `private`.
