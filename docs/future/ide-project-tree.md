# IDE project and workspace tree

**Status:** `Std.Toml` and `Std.Fs.Glob` completed on 2026-07-14. IDE session state (manifest load + in-memory root) completed. The IDE tree window (step 4) is next.

## Goal

When the FPAS IDE opens a `.fpasprj` project or `.fpasworkspace` workspace, it automatically adds a non-modal Turbo Vision window containing an expandable tree:

```text
workspace.fpasworkspace
└─ application.fpasprj
   └─ src
      ├─ main.fpas
      └─ ui
         └─ menu.fpas
```

The tree is display-only in this scope. It opens with the project or workspace and remains a desktop window until the application exits. Selecting a file, opening an editor, refresh, filtering, and arbitrary filesystem browsing are explicitly out of scope.

Project and workspace manifests remain TOML. Do not migrate them to JSON.

## Dependency order

Implement the work strictly in this order:

1. `Std.Toml` — complete
2. `Std.Fs.Glob` — complete
3. IDE session state — complete
4. IDE project/workspace tree window — next

Each item must land with its own current documentation, tests, and completed-plan status before beginning the next one.

## 1. `Std.Toml` — complete

Add a focused `Std.Toml` unit so FPAS code can parse TOML text returned by `Std.Fs.ReadText`.

### Required surface

- `TomlValue` representation for TOML scalar values, arrays, and tables.
- `Parse(Text: string): Result of TomlValue, string`.
- Table and array access sufficient for the IDE to read `[workspace]`, `[project]`, and `[sources]` fields.
- Diagnostics that identify malformed TOML and the expected TOML construct.

### Scope boundary

This is a TOML parser API, not a second project loader. It must not resolve workspace members, source patterns, project dependencies, or paths.

### Completion checks

- Unit registration, compiler lowering, bytecode, and runtime wiring are covered by focused Rust tests.
- FPAS regression tests under `tests/stdlib/toml/`.
- [`Std.Toml`](../pascal/std/text/toml.md) documents the implemented API.

## 2. `Std.Fs.Glob` — complete

Add a filesystem operation that resolves FPAS project source patterns such as `src/**/*.fpas` into actual files. `Std.Fs.ReadText` alone cannot expand `[sources].include` patterns, so this step is required before the IDE can build a real directory/file tree.

### Required surface

- `Glob(Pattern: string): Result of array of string, string`.
- Stable, deterministic path order.
- Only paths matching the supplied pattern are returned.
- Errors identify invalid patterns or filesystem failures.

### Scope boundary

Do not add a general-purpose file explorer, recursive directory listing API, watcher, or background refresh mechanism. The IDE needs glob expansion only.

### Completion checks

- Rust tests in `crates/fpas-std/src/fs.rs` cover ordinary patterns, recursive patterns, no matches, invalid patterns, plain file paths, and stable ordering.
- FPAS regression tests under `tests/stdlib/fs/`:
  - `fs_glob_flat_pattern_returns_sorted_paths_test.fpas`
  - `fs_glob_recursive_pattern_test.fpas`
  - `fs_glob_no_matches_returns_empty_array_test.fpas`
  - `fs_glob_invalid_pattern_returns_error_test.fpas`
- [`Std.Fs`](../pascal/std/host/fs.md) documents `Glob` and platform considerations.

### Implemented behavior (summary)

- Returns only regular files; directories are never included.
- Valid patterns with no matches return `Ok([])` rather than an error.
- A plain existing file path without glob metacharacters returns `Ok([Path])`.
- Returned path strings use `/` separators for deterministic cross-platform ordering.

## 3. IDE session state — complete

Keep a loaded workspace or project root in memory while the IDE runs. Opening another root replaces the previous one.

### Implemented layout

```text
apps/ide/src/workspace/
 ├── classify.fpas   path → OpenKind
 ├── model.fpas      LoadedProject, LoadedWorkspace, RootState
 ├── load.fpas       ReadText + Toml.Parse → model values
 └── session.fpas    ActiveRoot + open files, OpenPath coordinator
```

### Completion checks

- `OpenPath` loads `.fpasprj` / `.fpasworkspace` manifests instead of storing only the path string.
- Session accessors expose name, kind, members, include patterns, and main path.
- IDE tests under `apps/ide/tests/workspace/` cover load and session behavior.

## 4. IDE project/workspace tree window — next

Extend the FPAS IDE source so `File / Open` creates a tree window automatically after it successfully opens a project or workspace root.

### Data loading

- Read the chosen `.fpasprj` or `.fpasworkspace` with `Std.Fs.ReadText`.
- Parse the manifest with `Std.Toml.Parse`.
- For a workspace, resolve `workspace.members` relative to the workspace manifest and load each member project.
- For a project, read `project.main` and expand every `[sources].include` pattern with `Std.Fs.Glob`.
- Exclude paths matched by `[sources].exclude`; retain the program `main` even if it would otherwise be excluded from the source list.
- Group resolved relative paths into directory nodes, with directories before files and deterministic ordering.

### Window behavior

- Build a `Window` and attach it through `Desktop.Add`; do not use `Application.ExecView` or a modal `Dialog`.
- Use `Outline.New` with `OutlineNode` records. Project, directory, and workspace nodes start expanded; file nodes are leaves.
- Title the window with the opened root's display name and use a layout that works within the existing IDE menu and status chrome.
- Opening another project or workspace replaces the session root and opens the corresponding tree window. Do not add file activation behavior in this item.

### Tests and completion checks

- Add IDE tests for a project tree and workspace tree, including nested directories, deterministic ordering, and initial expanded state.
- Add a shell test proving that a successful `File / Open` for either root type creates the non-modal desktop window.
- Run the targeted IDE suite and the TUI outline tests, then the required workspace verification for the touched API layers.
- Move completed behavior into `docs/pascal/`; update this file so the next unfinished step is unambiguous.

## Handoff

The next implementation step is **4. IDE project/workspace tree window** after session-state tests pass.

Prerequisites are satisfied:

- [`Std.Toml`](../pascal/std/text/toml.md) — parse workspace and project manifests in FPAS.
- [`Std.Fs.Glob`](../pascal/std/host/fs.md) — expand `[sources].include` patterns in FPAS.

Do not re-open steps 1 or 2 unless a regression is found in docs or tests.
