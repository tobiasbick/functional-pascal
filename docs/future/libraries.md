# Libraries and Program Artifacts

This document records the agreed target architecture for compiled units,
compiled programs, and host-native application bundles. It intentionally
contains no implementation history, measurements, or optional roadmap ideas.

## Direction

- Units are compiled independently into reusable, relocatable objects.
- Programs are linked into reusable executable VM bytecode images.
- Sources and manifests remain authoritative build inputs.
- Derived artifacts are validated by content and compatibility identities, not
  file modification times.
- Normal commands rebuild missing, stale, incompatible, or corrupt artifacts
  automatically.
- The FPAS language, visibility rules, and unit startup semantics do not change.

## Projects and libraries

Libraries remain normal `kind = "library"` projects:

- `[dependencies].projects` references relative or absolute `.fpasprj` files;
- `[dependencies].workspace` references `project.name` values in the enclosing
  `.fpasworkspace`;
- `[exports].units` controls which units a dependent project may import;
- declaration-level `public` controls which declarations a unit exports;
- the workspace lists members but does not create implicit dependencies.

The ownership boundaries remain:

1. `fpas-project` loads projects and workspaces and resolves the unit graph.
2. `fpas-build` validates or builds compiled artifacts.
3. `fpas-linker` links the program object and reachable unit objects.
4. `fpas-vm` executes the final program image.

## Compiled units: `.fpascu`

Every `.fpas` unit has one source-adjacent `.fpascu` sidecar. The suffix means
Functional Pascal compiled unit.

A compiled unit contains:

- a versioned envelope and compatibility identities;
- the source-content and semantic-interface hashes;
- the ordered direct dependencies and their interface hashes;
- the serialized public semantic interface;
- relocatable bytecode and persistent constants;
- imports, exports, definitions, and relocations;
- deterministic unit startup code;
- source locations required for diagnostics.

The semantic interface is the incremental compilation boundary. An
implementation-only change rebuilds the changed unit and relinks programs, but
does not semantically rebuild consumers while the exported interface hash stays
unchanged. An interface change invalidates affected consumers transitively.

`fpas check`, `fpas run`, `fpas test`, and `fpas build` validate and reuse
compatible unit sidecars. Missing, stale, incompatible, or corrupt sidecars are
rebuilt from source.

`.fpascu` files are derived outputs:

- they live beside their source units;
- they are ignored by Git and source discovery;
- they are written atomically under a sidecar lock;
- they do not replace `.fpas` sources or `.fpasprj` manifests;
- obsolete formats are rebuilt rather than migrated.

Source-defined `Std.*` units use the same compiled-unit and linker pipeline as
project libraries. There is no special compilation path for `Std.Tui` or other
standard units.

## Compiled programs: `.fpascp`

A program project can produce one `.fpascp` file. The suffix means Functional
Pascal compiled program.

The default artifact is named from `project.name` and is written beside the
program's `.fpasprj` manifest:

```text
hello.fpasprj
hello.fpascp
src/
  main.fpas
```

A compiled program contains:

- a versioned envelope and compatibility identities;
- the compiler, bytecode, and compilation-option identities;
- the main program source hash;
- the ordered identities and object hashes of all linked units;
- the final linked instruction stream and persistent constant pool;
- the function table;
- source locations and a non-machine-specific source-path table;
- a payload hash.

The unit object hashes are part of the program identity. A reachable unit
implementation change therefore invalidates and relinks the program even when
the unit's public interface did not change.

When invoked through a project or workspace, sources and manifests remain
authoritative. A compatible `.fpascp` is reused; otherwise it is rebuilt and
replaced atomically. A `.fpascp` passed directly to `fpas run` is validated for
format and runtime compatibility and can run without source files.

`fpas check` does not need to publish a `.fpascp`. Explicit `fpas build` and
automatic `fpas run` are the program-artifact producers.

## CLI

Program builds use:

```text
fpas build [--executable] [--name <name>] [<file.fpasprj | file.fpasworkspace>]
```

The path may be omitted. Discovery follows the existing workspace-first,
single-project fallback used by other project-aware commands.

Without `--executable`:

- a program project produces or reuses its `.fpascp`;
- a library project eagerly produces or reuses its `.fpascu` files;
- a workspace builds its library units and produces one `.fpascp` per program
  member.

With `--executable`:

- the selected input must resolve to exactly one program;
- the command produces a host-native single-file application;
- `--name <name>` optionally supplies its application and output base name;
- without `--name`, a project uses `project.name` and a workspace uses
  `workspace.name`;
- `--name` without `--executable` is an error;
- the application name does not change the FPAS `program` declaration name;
- icons and platform-specific product metadata are outside this naming option.

Program execution uses:

```text
fpas run [<file.fpas | file.fpasprj | file.fpasworkspace | file.fpascp>] [-- <args>...]
```

- Running a project validates, rebuilds if necessary, and executes its
  `.fpascp`.
- Running a workspace requires exactly one `kind = "program"` member.
- Running a `.fpascp` executes the validated bytecode directly.
- Program arguments after `--` remain available through `Std.Args`.

All commands are non-interactive and idempotent. Repeating a successful build
with unchanged inputs reuses the existing compatible outputs.

## Workspace rules

`fpas build <workspace>` builds every member. Each program member owns its
`.fpascp` beside its own `.fpasprj`.

`fpas run <workspace>` and `fpas build --executable <workspace>` require exactly
one program member:

- zero program members is an actionable error;
- multiple program members is an actionable error that asks for a concrete
  `.fpasprj`;
- a single program member uses the workspace name as the default bundled
  application name.

The loaded workspace model therefore retains `workspace.name` in addition to
its resolved member projects.

## Host-native single-file applications

The single-file application combines:

```text
host-native fpas runner
+ compiled program bytecode
= application executable
```

The runner contains only what execution requires:

- `fpas-vm`;
- the native `fpas-std` intrinsic runtime;
- the program-image decoder and validator;
- diagnostic support required at runtime.

Parser, semantic analysis, compiler, project loading, unit building, and linking
remain build-time components and are not included in the application.

The build uses the runner for the current host:

- Windows produces a PE executable named `<name>.exe`;
- Linux produces an executable ELF file named `<name>` and sets its executable
  permission;
- no cross-compilation or `--target` option is provided.

The native runner is built once for the host. Packaging copies that runner and
attaches the validated `.fpascp` payload, the application name, and a fixed
footer. The application name belongs to the native bundle, so the same
`.fpascp` can be bundled under different names without relinking. Packaging does
not invoke the Rust compiler separately for every FPAS application.

The resulting application runs without `fpas`, `.fpascu` files, source files,
project manifests, or the source standard library. Files intentionally loaded
by the FPAS program through APIs such as `Std.Fs` remain normal external
application data.

## Validation and publication

Both `.fpascu` and `.fpascp` formats:

- use explicit magic and format versions;
- record compiler, bytecode, options, source, and dependency identities;
- bound strings, counts, and payload sizes while decoding;
- reject truncation, trailing data, invalid tags, invalid indices, unsupported
  runtime values, and inconsistent hashes;
- publish through a same-directory temporary file and atomic replacement;
- never fall back to an implicit global cache when publication fails.

Host-native application packaging validates the program payload before
publication and writes the final executable atomically.

## Boundaries

This design does not add:

- new FPAS syntax or unit initialization/finalization constructs;
- a `.fpaslib` container;
- source-less library distribution;
- a global artifact cache;
- a package registry, lockfile, or version solver;
- cross-compilation;
- native-code generation for FPAS programs.

The bundled application remains a native VM/runtime executing FPAS bytecode.
