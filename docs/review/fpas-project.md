# `fpas-project` review follow-up

Classification: project/workspace loading and Unit graph resolution. Preserve the current path/workspace and source-adjacent sidecar model.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| PROJECT-01 | P1 | `crates/fpas-project/src/unit_graph/mod.rs:301`, `src/unit_graph/resolve.rs:134` | Every missing `Std.*` Unit is classified as intrinsic and skipped. `uses Std.Typo` succeeds instead of diagnosing an unknown Unit. | Skip only names present in the authoritative known-intrinsic set. Source-defined `Std.*` Units remain ordinary graph nodes. | Unknown root/transitive `Std.DoesNotExist` fails; known intrinsic and source-defined Std Units still work. |
| PROJECT-02 | P1 | `crates/fpas-project/src/unit_graph/mod.rs:209`, `src/unit_graph/model.rs:101` | Graph loading trusts sidecar dependencies after checking only source hash, not compiler/format/dependency compatibility. A decodable old sidecar can provide stale reachability. | Apply the same full compatibility identity as the build path before using sidecar metadata; otherwise parse source. | Matching source hash but wrong compiler identity or stale dependencies is ignored and rebuilt/reparsed. |
| PROJECT-03 | P2 | `crates/fpas-project/src/model.rs:90,106`, `src/unit_graph/parsed.rs:32` | Source origin/export trust uses exact `PathBuf` keys. Canonical, `..`, or symlink aliases can turn dependency sources into `Own` and bypass private export rules. | Normalize paths consistently at insertion and lookup, or use stable same-file identities. | Lexical/canonical/symlink aliases preserve dependency origin, exports, and Std trust. |
| PROJECT-04 | P2 | `crates/fpas-project/src/loading/own.rs:103`, `src/workspace/resolve.rs:11,32` | Lower confidence: a relative manifest basename has an empty parent and can break upward workspace discovery. | Normalize the public manifest path to an absolute path before deriving its directory. | Load a project by basename from its directory and resolve the enclosing workspace. |
| PROJECT-05 | P3 | `crates/fpas-project/src/loading/own.rs:267`, `src/loading/exports.rs:13`, `src/loading/parse_cache.rs:1` | Two validations accept `ParsedSourceCache` but ignore it and reread headers, complicating the pipeline and defeating the cache contract. | Route validation through the cache or remove the unused parameter and define a separate header-cache layer. | Count parse/cache misses across main, export, and Unit validation. |

## Implementation notes

PROJECT-01 is a correction to existing known-Std behavior, not a language extension. PROJECT-02 depends on explicit compatibility identities from `fpas-build`, `fpas-program`, and `fpas-unit`. Do not introduce registries, package managers, or a global artifact cache.
