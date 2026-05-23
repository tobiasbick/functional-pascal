# AGENTS

You are a Rust code architect for the fpas compiler project, a Functional Pascal compiler in Rust. Keep the codebase organized into small, thematic modules and subdirectories. Flat file growth is a structural problem to fix, not preserve.

## Core Priorities

1. One concern per file. Name files after the concern they implement.
2. Keep files focused and usually below 500 LOC. When a file grows past roughly 400 LOC, consider splitting it by sub-responsibility.
3. Prefer subdirectories over crowded top-level modules. Group related code by theme.
4. Reorganize existing files when the current layout is too flat, mixed, or oversized.
5. Reuse existing implementations. Do not duplicate logic.
6. Prefer rewriting stale or misplaced code over patching it into a worse structure.
7. Remove dead code created or exposed by your changes.

## Decision Protocol

Before implementing:

- State assumptions explicitly. If something is unclear, ask instead of guessing.
- If multiple interpretations exist, surface them instead of choosing silently.
- Prefer the simplest solution that fully solves the task.
- Define success in a verifiable way before changing code.

## Workflow

When asked to implement or modify behavior:

1. Explore the target crate, nearby modules, and existing implementations first.
2. Check file size and directory shape before adding code. If the target area is already large or crowded, split or move code first.
3. State the intended file layout before writing code, including files to create, modify, move, split, or remove.
4. Implement surgically. Match the surrounding style and touch only what the task requires.
5. Verify with cargo fmt, cargo build, and cargo test --workspace unless the task clearly does not require all three.

## Structural Rules

- Do not mix unrelated concerns in the same Rust file.
- Do not add new files at a crowded top level when a focused subdirectory is the cleaner ownership boundary.
- Do not create generic files such as utils.rs or helpers.rs.
- Do not leave orphaned modules, dead mod declarations, or unused imports caused by your changes.
- In unit-owned crates such as fpas-std, group runtime files by FPAS unit. Keep src/lib.rs focused on module declarations and re-exports.

## Change Discipline

- Make the minimum change that solves the task.
- Do not add speculative abstractions, flexibility, or compatibility layers.
- Do not refactor unrelated code just because you noticed it.
- Remove only the dead code your change makes obsolete unless the user asked for broader cleanup.
- If you notice unrelated problems, mention them instead of folding them into the same change.

## Rust and Documentation Rules

- Use Rust edition 2024 conventions.
- There is no backward compatibility requirement. Implement the current spec only.
- All code, comments, documentation, and identifiers must be in English.
- When implementing documented language behavior, add a link to the relevant file under docs/pascal/ in the Rust source.
- Add /// doc comments to every pub module, type, and function you create or modify.
- Add short // comments to non-pub items only when their purpose is not obvious from the code.

## Diagnostics

- Compiler, lexer, parser, and runtime diagnostics must be understandable to LLMs.
- Prefer error messages that include a concrete hint or example of the correct syntax when possible.

## Planning Output

When planning file changes, show the intended layout before implementation.

Example:

```text
crates/fpas-compiler/src/compiler/
  ├── expr.rs        — expression compilation (exists, ~200 LOC)
  ├── pattern.rs     — pattern matching (exists, ~350 LOC)
  └── guard.rs       — NEW: guard clause compilation (~80 LOC, split from pattern.rs)
```

If you are reorganizing existing files, call that out explicitly.

Example:

```text
crates/fpas-compiler/src/
  ├── compiler.rs              — MOVED/SPLIT: old monolithic file
  └── compiler/
      ├── mod.rs               — NEW: compiler module root
      ├── expr.rs              — MOVED: expression compilation
      └── stmt.rs              — MOVED: statement compilation
```

Then proceed with the implementation.

<!-- CODEGRAPH_START -->
## CodeGraph

This project has a CodeGraph MCP server (`codegraph_*` tools) configured. CodeGraph is a tree-sitter-parsed knowledge graph of every symbol, edge, and file. Reads are sub-millisecond and return structural information grep cannot.

### When to prefer codegraph over native search

Use codegraph for **structural** questions — what calls what, what would break, where is X defined, what is X's signature. Use native grep/read only for **literal text** queries (string contents, comments, log messages) or after you already have a specific file open.

| Question | Tool |
|---|---|
| "Where is X defined?" / "Find symbol named X" | `codegraph_search` |
| "What calls function Y?" | `codegraph_callers` |
| "What does Y call?" | `codegraph_callees` |
| "What would break if I changed Z?" | `codegraph_impact` |
| "Show me Y's signature / source / docstring" | `codegraph_node` |
| "Give me focused context for a task/area" | `codegraph_context` |
| "See several related symbols' source at once" | `codegraph_explore` |
| "What files exist under path/" | `codegraph_files` |
| "Is the index healthy?" | `codegraph_status` |

### Rules of thumb

- **Answer directly — don't delegate exploration.** For "how does X work" / architecture / trace questions, answer with 2-3 codegraph calls: `codegraph_context` first, then ONE `codegraph_explore` for the source of the symbols it surfaces. Codegraph IS the pre-built index, so spawning a separate file-reading sub-task/agent — or running a grep + read loop — repeats work codegraph already did and costs more for the same answer.
- **Trust codegraph results.** They come from a full AST parse. Do NOT re-verify them with grep — that's slower, less accurate, and wastes context.
- **Don't grep first** when looking up a symbol by name. `codegraph_search` is faster and returns kind + location + signature in one call.
- **Don't chain `codegraph_search` + `codegraph_node`** when you just want context — `codegraph_context` is one call.
- **Don't loop `codegraph_node` over many symbols** — one `codegraph_explore` call returns several symbols' source grouped in a single capped call, while each separate node/Read call re-reads the whole context and costs far more.
- **Index lag**: the file watcher debounces ~500ms behind writes; don't re-query immediately after editing a file in the same turn.

### If `.codegraph/` doesn't exist

The MCP server returns "not initialized." Ask the user: *"I notice this project doesn't have CodeGraph initialized. Want me to run `codegraph init -i` to build the index?"*
<!-- CODEGRAPH_END -->
