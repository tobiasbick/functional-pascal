# Performance candidates

These findings identify work performed by the current source. No elapsed-time improvement or production impact has been measured. Proposed benchmarks belong to follow-up implementation work.

## P1. Exclusion filtering repeatedly resolves the same paths

Priority P2. `crates/fpas-project/src/paths.rs:20` computes the included file's canonical key once, then calls `canonical_or_original` for each excluded candidate until a match is found. That helper at line 247 calls `fs::canonicalize`. Exclusion expansion already computed canonical keys for deduplication, but discards that set and returns original paths.

For N included files and M exclusions, the filter can perform N times M filesystem canonicalizations of exclusions. Large generated-source exclusions can dominate project loading, particularly on slower filesystems.

Retain canonical exclusion keys in a `HashSet<PathBuf>`, and perform one lookup per canonical included path. Preserve the existing missing-path fallback and case-alias behavior. Existing include/exclude tests cover behavior; add an alias case and verify canonicalization counts through a focused seam or measure a large fixture. Compare N and 2N files with proportional exclusions.

## P2. Register allocation repeatedly rebuilds its occupancy tree

Priority P2. `crates/fpas-compiler/src/bytecode/allocation/mod.rs:76` builds a new `BTreeSet` for every non-coalesced result from all active temporaries and all locals. `lowest_free` at line 202 starts searching at `next_fixed`, above every fixed local register. Adding those locals to the set therefore cannot affect the selected temporary register.

With T temporary-producing instructions and L fixed locals, this introduces at least T times L insert attempts even when temporary pressure is small. `active.retain` also rescans all live temporaries at each instruction. A function with many locals or long-lived values magnifies the work.

First remove the provably irrelevant fixed-local insertions. Then measure whether maintaining occupied/free temporary registers incrementally is worthwhile. Preserve deterministic lowest-free allocation, last-use semantics, coalescing, call windows, and register-limit diagnostics. Existing allocation tests in the neighboring `tests.rs` are the behavioral baseline. Use generated IR with separately varied local count, instruction count, and live temporary count to identify the dominant cost.

## P3. A semantic cache hit still reads the closed source tree

Priority P2. `crates/fpas-language-service/src/analysis/service.rs:157` gathers all project snapshots before checking the semantic fingerprint. `DocumentStore::snapshot` in `document/store.rs:139` reads a closed file with `fs::read_to_string` before comparing it with a cached snapshot. Thus repeated queries with no source changes still read and compare every closed project source, including composed standard-library sources.

`analyze_document_diagnostics` also gathers project snapshots before calling `analyze_document`, which gathers them again. This duplicates closed-file reads on that path. `fork_for_queries` shares parsed snapshots but does not eliminate these disk reads.

Start by avoiding the duplicate snapshot gathering within one diagnostic request. A broader cache change needs an explicit freshness policy: current direct callers observe disk changes without requiring a watched-file event. Do not substitute indefinite cached contents and silently lose that behavior. A request-scoped snapshot batch can reduce repeated work while retaining freshness at request boundaries.

Measure bytes read and filesystem calls for repeated hover/diagnostic requests over a project with many closed units. Cover direct disk mutation without a watcher, watcher invalidation, open-buffer precedence, and concurrent query forks before changing cache reuse.

## P4. Avoidable copies survive otherwise borrowed interfaces

Priority P3. Two concrete cases:

- `crates/fpas-build/src/engine.rs:157` creates deep-cloned direct interfaces before `Backend::load` determines whether the sidecar is reusable. Those interfaces are only used in the compilation branch at line 184. Move their construction into that branch. `InterfaceRegistry::finish` in `engine/interfaces.rs:76` also copies every interface into a second ownership structure; assess shared ownership only if measurements justify changing that internal API.
- `crates/fpas-std/src/intrinsic_args.rs:84` converts a borrowed `SharedStr` into a new `String`. `parse.rs:18`, `conv.rs:30`, and path queries use this helper despite only reading the text. `expect_str` already provides the borrowing alternative. Preserve owned values where host APIs or escaping results actually require them.

The existing parse cache also clones complete ASTs on cache hits at `crates/fpas-project/src/loading/parse_cache.rs:39`. All inspected callers pass source ID zero, so the cache's omitted source ID is not reported here as a current correctness bug.

Validate copy removal with allocation counts for successful parsing and a warm build with many imports. Existing functional tests should remain unchanged. Do not equate cheap `Arc` clones with cloning AST vectors or string contents.

## P5. Linker sort-key evaluation scans definition tables repeatedly

Priority P3. `crates/fpas-linker/src/plan/layouts.rs:39` and line 56 use `sort_by_key` with functions returning newly owned canonical names. `SymbolTable::canonical_target_name` in `plan/symbols.rs:154` linearly searches the object's definitions and clones the selected name. Sorting invokes key extraction repeatedly, followed by another extraction in the subsequent assignment loop.

For K layouts and D definitions, name lookup contributes approximately O(K log K times D) work for general unsorted input, plus temporary strings. Precompute each layout's canonical name once, sort the stored name/index pairs, and reuse the name during assignment. A reverse definition lookup could help other consumers, but is unnecessary if this one local change resolves the measured cost.

Use fixtures with many distinct layouts and unrelated definitions. Keep deterministic layout IDs, equivalent-layout coalescing, visibility checks, and enum variant order covered by the existing linker tests.
