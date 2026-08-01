# `fpas-sema` review follow-up

Classification: semantic analysis and internal API structure. Fixes should enforce existing case-insensitive names and pattern typing.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| SEMA-01 | P1 | `crates/fpas-sema/src/check/stmt/control_flow/if_case/mod.rs:96`, `src/tests/stmt/flow.rs:102` | Each label in a multi-pattern arm analyzes the same expression with its own binding types. Incompatible payload types overwrite the same expression metadata; last label wins. | Compare all label binding signatures first, require matching names and compatible types, then analyze the body once using one shared signature. | `Result<Integer,String>` with common binding, differing binding names, and duplicate names inside a data-enum pattern. |
| SEMA-02 | P2 | `crates/fpas-sema/src/check/expr/mod.rs:207,264`, `src/check/decl/vars.rs:82` | Anonymous literals and record updates do not reject duplicate fields; typed literals compare exact spelling rather than case-insensitive identity. | Use one shared duplicate-field validator based on `canonical_symbol_name` for literals and updates. | Exact and case-only duplicates in anonymous/typed literals and `with` updates. |
| SEMA-03 | P3 | `crates/fpas-sema/src/lib.rs:62-65`, `src/interface.rs:43` | Main analysis API returns an undocumented thirteen-element tuple of metadata maps. Callers can easily misorder additions. | Return a documented named `AnalysisMetadata` structure, coordinated with compiler consumers. | API regression covers every metadata field; migrate compiler/unit-object call sites together. |
| SEMA-04 | P3 | `crates/fpas-sema/src/interface.rs:1`, `src/check/decl/types/records.rs:1` | `interface.rs` exceeds 1,100 LOC and mixes analysis, installation, exports, conversions, and tests; `records.rs` exceeds 700 LOC and mixes fields, methods, static routines, and receiver checks. | Split into thematic submodules such as `interface/{analysis,install,exports,conversion}` and `records/{fields,methods,receiver}` while keeping behavior unchanged. | Existing suite only plus module-level tests moved with their ownership; no new behavior. |

## Implementation notes

Implement SEMA-01 before compiler work that relies on expression metadata. SEMA-03 and COMPILER-04 should be one refactor slice. Record `docs: unchanged` for internal corrections unless current user-facing docs make an incorrect claim.
