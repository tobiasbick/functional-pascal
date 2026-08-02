# `fpas-project` review follow-up

Classification: project/workspace loading and Unit graph resolution. Preserve the current path/workspace and source-adjacent sidecar model.
Status: PROJECT-01 through PROJECT-05 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| PROJECT-01 | P1 | `crates/fpas-project/src/unit_graph/mod.rs:301`, `src/unit_graph/resolve.rs:134` | Every missing `Std.*` Unit is classified as intrinsic and skipped. `uses Std.Typo` succeeds instead of diagnosing an unknown Unit. | Skip only names present in the authoritative known-intrinsic set. Source-defined `Std.*` Units remain ordinary graph nodes. | Unknown root/transitive `Std.DoesNotExist` fails; known intrinsic and source-defined Std Units still work. |
| PROJECT-02 | P1 | `crates/fpas-project/src/unit_graph/mod.rs:209`, `src/unit_graph/model.rs:101` | Graph loading trusts sidecar dependencies after checking only source hash, not compiler/format/dependency compatibility. A decodable old sidecar can provide stale reachability. | Apply the same full compatibility identity as the build path before using sidecar metadata; otherwise parse source. | Matching source hash but wrong compiler identity or stale dependencies is ignored and rebuilt/reparsed. |
| PROJECT-03 | P2 | `crates/fpas-project/src/model.rs:90,106`, `src/unit_graph/parsed.rs:32` | Source origin/export trust uses exact `PathBuf` keys. Canonical, `..`, or symlink aliases can turn dependency sources into `Own` and bypass private export rules. | Normalize paths consistently at insertion and lookup, or use stable same-file identities. | Lexical/canonical/symlink aliases preserve dependency origin, exports, and Std trust. |
| PROJECT-04 | P2 | `crates/fpas-project/src/loading/own.rs:103`, `src/workspace/resolve.rs:11,32` | Lower confidence: a relative manifest basename has an empty parent and can break upward workspace discovery. | Normalize the public manifest path to an absolute path before deriving its directory. | Load a project by basename from its directory and resolve the enclosing workspace. |
| PROJECT-05 | P3 | `crates/fpas-project/src/loading/own.rs:267`, `src/loading/exports.rs:13`, `src/loading/parse_cache.rs:1` | Two validations accept `ParsedSourceCache` but ignore it and reread headers, complicating the pipeline and defeating the cache contract. | Route validation through the cache or remove the unused parameter and define a separate header-cache layer. | Count parse/cache misses across main, export, and Unit validation. |

## Implementation notes

PROJECT-01 is a correction to existing known-Std behavior, not a language extension. PROJECT-02 depends on explicit compatibility identities from `fpas-build`, `fpas-program`, and `fpas-unit`. Do not introduce registries, package managers, or a global artifact cache.

## Implementation record

- PROJECT-01 adds `STD_UNITS_INTRINSIC` as the authoritative runtime-backed subset in `fpas-std`.
  Missing entries are skipped only when they occur in that set. Source-defined `Std.Tui`,
  `Std.Version`, and manifest-owned `Std.*` units remain ordinary graph nodes; unknown root and
  transitive `Std.*` imports now fail.
- PROJECT-02 removes sidecar metadata from project loading and Unit graph construction. Current
  source declarations exclusively define Unit names and dependency edges. Full sidecar identity
  validation and payload reuse remain owned by `fpas-build`, after the authoritative graph exists.
  The obsolete lazy sidecar-backed `UnitNode` state was removed.
- PROJECT-03 canonicalizes internally inserted source identities and makes all public link-metadata
  lookups same-file aware. Lexical, canonical, and symlink aliases therefore preserve library
  origins, export policy, and trusted standard-library provenance.
- PROJECT-04 converts public manifest inputs to absolute paths without exposing Windows verbatim
  canonical paths. Loading a project by basename can discover and resolve its enclosing workspace.
- PROJECT-05 routes main, export, user-Unit, trusted standard-library, and intrinsic-collision
  validation through one canonical `ParsedSourceCache`. Each distinct source is parsed once per
  project load.
- `docs/pascal/program-structure/units.md` now distinguishes authoritative project graph parsing
  from compiled interface/object reuse in the build stage. FPAS syntax and semantics are unchanged.

## Verification

- `cargo test -p fpas-project --locked` — 22 unit tests and 32 integration tests passed, including
  10 project-integrity regressions; doc tests passed.
- `cargo test -p fpas-build --locked` — all 23 unit and integration tests passed.
- `cargo test -p fpas-std std_units::units::tests::intrinsic_units_exclude_source_defined_units
  --locked` — passed.
- `cargo clippy -p fpas-project -p fpas-std -p fpas-build --all-targets --locked -- -D warnings`
  — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
