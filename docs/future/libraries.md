# Libraries: Compiled-Unit Architecture

Status: implemented architecture record with remaining follow-up work.

The normative description of current behavior is:

- [`units.md`](../pascal/program-structure/units.md) for unit imports and
  source-adjacent `.fpascu` files;
- [`projects.md`](../pascal/program-structure/projects.md) for library projects,
  dependencies, and `[exports].units`;
- [`workspaces.md`](../pascal/program-structure/workspaces.md) for workspace
  dependency resolution;
- [`cli.md`](../pascal/program-structure/cli.md) for automatic builds;
- [`Std.Test`](../pascal/std/testing/test.md) for compiled-unit reuse during test
  execution.

This page preserves the decisions and architectural intent behind that behavior.
It is not a second user-facing specification.

## Fixed decisions

These decisions are implemented and remain the baseline for later library work:

- **Every unit is compiled independently.** FPAS no longer compiles libraries by
  cloning their declarations into one consumer `Program` AST.
- **Source-adjacent unit objects use `.fpascu`.** Compiling `Example.fpas`
  produces `Example.fpascu` beside the source. The suffix means Functional
  Pascal compiled unit.
- **Normal commands build automatically.** `fpas check`, `fpas run`, and
  `fpas test` rebuild missing, stale, incompatible, or corrupt unit objects as
  needed. A separate `fpas build` command is not required.
- **Sources and manifests are authoritative.** `.fpascu` files are derived,
  replaceable build outputs. They do not introduce another dependency syntax.
- **Compatibility is explicit.** A unit object records compiled-unit format,
  compiler, bytecode, option, source, interface, and dependency identities. A
  mismatch invalidates the object instead of attempting best-effort loading.
- **Interfaces are the incremental boundary.** An implementation-only change
  rebuilds its unit and relinks consumers, but does not semantically rebuild
  consumers while the exported interface hash remains unchanged.
- **Linking is reachability-based.** The final bytecode image includes only unit
  objects reachable from the program's `uses` graph.
- **Existing visibility rules remain.** Unit `private` declarations and project
  `[exports].units` continue to define the public boundaries.
- **No new unit lifecycle syntax exists.** Existing top-level constant and
  variable initializers form unit startup code and execute in deterministic
  dependency order.
- **The initial artifact is one sidecar per unit.** There is no `.fpaslib`
  container, project-local artifact directory, global cache, registry, lockfile,
  or semver dependency resolver.
- **There is no compatibility promise for obsolete `.fpascu` formats.** The
  source is rebuilt with the current compiler.

## Project dependency model

Libraries remain normal FPAS projects:

- `kind = "library"` declares a library project containing units and no `main`;
- `[dependencies].projects` references relative or absolute `.fpasprj` files;
- `[dependencies].workspace` references a member by `project.name` in the
  enclosing `.fpasworkspace`;
- `[exports].units` optionally lists units visible to dependent projects;
- unlisted units remain usable inside their owning library but are inaccessible
  across the project boundary.

Project resolution, unit compilation, and program linking are deliberately
separate concerns:

1. `fpas-project` loads projects and constructs the reachable unit graph.
2. `fpas-build` validates or builds independent unit objects.
3. `fpas-linker` combines the program object and reachable unit objects into the
   VM's final `Chunk`.

## `.fpascu` contents

Each compiled unit contains a versioned envelope with three logical areas.

### Identity and validation metadata

- canonical and display unit identity;
- source-content hash;
- semantic-interface hash;
- compiler identity;
- bytecode version;
- compiled-unit format version;
- semantic/code-generation option hash;
- ordered direct dependencies and their expected interface hashes.

Consequently, a bytecode-format change makes an existing `.fpascu`
incompatible. The same applies to a compiler identity change. A source, option,
unit-name, or dependency-interface change makes it stale. Missing, stale,
incompatible, and corrupt objects are rebuilt from source.

Validation uses recorded identities and hashes, not file modification times.
The decoder bounds strings, dependency counts, and payload sizes and rejects
bad magic, unsupported versions, truncation, inconsistent hashes, invalid tags,
and trailing data.

### Semantic interface

The serialized interface contains the public information required to analyze a
consumer without loading the dependency implementation AST:

- constants and compile-time values;
- variables and mutability;
- functions, procedures, parameters, generics, and constraints;
- records, fields, methods, properties, events, and relevant defaults;
- enums, variants, and associated data;
- aliases and composed types;
- canonical symbol ownership and visibility information.

Private bodies and source spans do not affect the interface hash unless they
change exported observable information.

### Relocatable implementation

The implementation payload contains:

- relocatable bytecode and local constant data;
- function, global, import, and export definitions;
- relocations for addresses and pool indices;
- startup code for top-level declaration initializers;
- original source locations needed by diagnostics.

The executable `Chunk` is still the final fully bound program image. A
`.fpascu` is a relocatable input to that image, not an executable chunk by
itself.

## Build and invalidation model

Units are processed in stable dependency order. For each unit, the build
pipeline:

1. derives the adjacent path with the `.fpascu` extension;
2. compares the recorded identity with the source, compiler, options, and
   already resolved dependency interfaces;
3. decodes and reuses the semantic interface and object when everything
   matches;
4. otherwise parses and independently analyzes the source;
5. emits a new interface and relocatable object;
6. validates and publishes the completed sidecar;
7. links the reachable objects with the program object.

An exported interface change invalidates direct consumers and propagates
through their rebuilt interface identities. A private implementation change
does not cause semantic consumer recompilation when the public interface hash
is unchanged, but the final program is relinked with the changed object.

Build events and counters record parsing, interface analysis, implementation
analysis, compilation, sidecar reuse, and relinking. Regression tests use those
events rather than timing to prove reuse.

## Sidecar lifecycle

`.fpascu` files:

- live beside their matching `.fpas` source;
- are ignored by Git and excluded from source discovery, formatting, and test
  discovery;
- are written through a same-directory temporary file and validated before
  publication;
- use a sidecar lock so concurrent commands do not observe partial objects;
- retain the source path for diagnostics without putting absolute
  machine-specific paths into deterministic identity content.

A read-only source directory can use a compatible existing sidecar. If a
rebuild is required and the adjacent sidecar cannot be written, the build
reports the source and artifact paths instead of redirecting the object to an
implicit cache.

Plain single-file programs without imported project or standard-library units
do not produce a compiled-unit sidecar for the program itself.

## Standard library and `Std.Tui`

Source-defined `Std.*` units use the same graph, interface, object, sidecar, and
linker pipeline as project libraries. Distribution scripts precompile sidecars
for the bundled standard-library sources.

`--std-lib <directory>` remains a complete source override. Sidecars are reused
only when their recorded source and compatibility identities match the selected
directory.

`Std.Tui` is the large acceptance workload: its focused internal units compile
independently, unchanged internal objects are reused, and consumers receive the
same behavior and source diagnostics as a clean source build.

## Verification snapshot

The implementation is covered by positive, negative, edge, corruption,
invalidation, linker, CLI/project, standard-library, and end-to-end FPAS tests.
The recorded isolated Windows debug measurement for the standard library was:

- cold build: 48 compiled, 0 reused, approximately 712 ms;
- warm build: 0 compiled, 48 reused, approximately 150 ms.

These measurements are contextual performance evidence. Deterministic build
counters are the correctness proof for reuse.

## Remaining library work

The compiled-unit architecture is complete. Potential later work is deliberately
separate:

- finer per-symbol project export and re-export tables beyond unit-level
  `[exports].units` and declaration-level `private`;
- an explicit `fpas build` convenience command for eager compilation;
- an explicit `fpas clean` workflow for derived sidecars and stale lock files;
- an optional project-level `.fpaslib` container or index without collapsing
  independently reusable unit objects;
- an optional shared artifact cache with explicit, predictable placement;
- source-less library distribution, if a stable compatibility policy is ever
  desired;
- package management, registries, lockfiles, and version solving as independent
  product decisions;
- finer per-symbol dead-code elimination inside a reachable unit.

None of these extensions should weaken source authority, deterministic
validation, unit/project visibility, or the separate compilation boundary.
