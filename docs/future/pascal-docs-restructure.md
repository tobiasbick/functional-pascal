# Future: Pascal documentation restructure

Plan to replace the flat numbered chapters (`01-overview.md` … `11-stdlib.md`) with **topic areas** (directories) and **small, themed pages** — similar to [Microsoft Learn](https://learn.microsoft.com/) language reference layout.

**Status:** in progress — `language/types/` migrated (Phase 1 scaffold + types split done).

**Normative spec today:** [`docs/pascal/`](../pascal/) (flat numbered files).

---

## Goals

1. **Discoverability** — find topics by name (`types/records.md`) instead of chapter number (`05-types.md`).
2. **Smaller pages** — one concern per file; easier reviews and clearer ownership (aligns with `AGENTS.md`).
3. **Consistency** — language spec matches the pattern already used in [`docs/pascal/std/`](../pascal/std/) (one unit / topic per file).
4. **Preserve learning path** — root [`docs/pascal/README.md`](../pascal/README.md) keeps an ordered “Start here” list for newcomers.
5. **Stable anchors** — area `README.md` hubs and predictable filenames so Rust `///` links and cross-references stay maintainable.

## Non-goals

- Rewriting spec content (move and split only, light edits for boundaries and cross-links).
- Changing [`docs/specs/grammar.ebnf`](../specs/grammar.ebnf) location or role.
- Restructuring [`docs/pascal/std/`](../pascal/std/) (already topic-based).
- Adding redirects or a doc build tool (plain Markdown in-repo only).

---

## Target layout

```text
docs/pascal/
  README.md                          # Hub: principles, learning path, area index
  getting-started/
    README.md                        # Area hub
    overview.md                      # Philosophy, design principles
    hello-world.md                   # Minimal program
    first-program.md                 # Slightly richer tour (from 01-overview)
    keywords.md                      # Reserved keyword list
  language/
    README.md                        # Area hub + grammar link
    basics/
      README.md
      primitive-types.md
      variables.md
      constants.md
      number-literals.md
      operators.md
      comments.md
      local-variables.md
      arrays-intro.md                # Syntax / literals only (see types/arrays.md)
    control-flow/
      README.md
      if-then-else.md
      case-of-intro.md               # Scalar case; links to pattern-matching/
      for-loops.md
      for-in.md
      while-repeat.md
      break-continue.md
    functions/
      README.md
      declarations.md
      parameters.md
      mutable-parameters.md
      function-types.md
      first-class.md
      nested.md
      mutual-recursion.md
      generic-routines.md
      early-return.md
    types/
      README.md
      records.md
      record-methods.md
      record-update.md
      enums.md
      arrays.md
      dictionaries.md
      type-aliases.md
      result-option-types.md         # Type forms only; behavior → error-handling/
      generics.md
    pattern-matching/
      README.md
      scalar-labels.md
      ranges.md
      enum-patterns.md
      result-option-patterns.md
      guards.md
      else-branch.md
      exhaustiveness.md
    error-handling/
      README.md
      result.md
      option.md
      try.md
      panic.md
      combinators.md                 # Map / AndThen / OrElse overview + std links
    concurrency/
      README.md
      go.md
      task-handles.md
      fork-join.md
      scheduling.md                  # Cooperative yield, main thread (from 08)
  program-structure/
    README.md
    units.md
    visibility.md                    # private / public, Std namespace
    projects.md                      # .fpasprj, kinds, dependencies, exports
    workspaces.md
    cli.md                           # fpas, check, test, fmt discovery
  std/                               # unchanged
    README.md
    …                                # existing per-unit pages
  tools/
    README.md
    fmt-style.md                     # moved from docs/pascal/fmt-style.md
```

**Formal grammar** stays at [`docs/specs/grammar.ebnf`](../specs/grammar.ebnf); every `language/` area hub links to the relevant grammar rules.

---

## Source mapping (current → planned)

| Current file | Lines (approx.) | Planned destination |
|--------------|-----------------|---------------------|
| `01-overview.md` | 73 | `getting-started/overview.md`, `hello-world.md`, `first-program.md`, `keywords.md` |
| `02-basics.md` | 195 | `language/basics/*` (split by `##` sections) |
| `03-control-flow.md` | 134 | `language/control-flow/*` |
| `04-functions.md` | 166 | `language/functions/*` |
| `05-types.md` | 334 | `language/types/*` (**largest split**) |
| `06-pattern-matching.md` | 214 | `language/pattern-matching/*` |
| `07-error-handling.md` | 121 | `language/error-handling/*` |
| `08-concurrency.md` | 87 | `language/concurrency/*` (may stay 3–4 files) |
| `09-units.md` | 91 | `program-structure/units.md`, `visibility.md` |
| `10-projects.md` | 204 | `program-structure/projects.md`, `workspaces.md`, `cli.md` |
| `11-stdlib.md` | 125 | Fold into `std/README.md` hub (overview table already duplicated) |
| `fmt-style.md` | — | `tools/fmt-style.md` |

After migration, **delete** the numbered top-level files (`01-` … `11-`, root `fmt-style.md`).

---

## Content boundaries (avoid duplication)

| Topic | Primary home | Secondary (link only) |
|-------|--------------|------------------------|
| Array / dict **syntax** in expressions | `basics/arrays-intro.md` | `types/arrays.md`, `types/dictionaries.md` |
| Array / dict **type semantics** | `types/arrays.md`, `types/dictionaries.md` | basics intro |
| `case of` on scalars | `control-flow/case-of-intro.md` | `pattern-matching/` |
| Guards, enum patterns, exhaustiveness | `pattern-matching/` | control-flow intro |
| `Result` / `Option` **type syntax** | `types/result-option-types.md` | error-handling |
| `try`, `panic`, combinator usage | `error-handling/` | `std/result.md`, `std/option.md` |
| `go` / `task` language rules | `concurrency/` | `std/task.md` |
| Per-routine API (`Wait`, `Map`, …) | `std/*.md` | language areas |

Each split page ends with a short **See also** block (2–4 links), not repeated prose.

---

## Page template

Every themed page:

```markdown
# <Topic>

<One-sentence summary.>

Formal syntax: [`grammar.ebnf`](../../specs/grammar.ebnf) (`<rule-names>`).

## …

## See also

- …
```

Area `README.md` files list child pages in a table (title + one-line description), mirroring [`docs/pascal/README.md`](../pascal/README.md) today.

---

## Migration phases

### Phase 1 — Scaffold (no link breakage)

1. Create directory tree and area `README.md` hubs with tables of contents (stubs OK).
2. Add the new learning path to root `docs/pascal/README.md` pointing at **both** old and new paths, or keep old paths until Phase 3.

### Phase 2 — Move and split content

1. **`language/types/`** — split `05-types.md` first (highest line count).
2. **`language/basics/`** — split `02-basics.md`.
3. **`program-structure/`** — split `10-projects.md`, move `09-units.md`.
4. **`language/functions/`**, **`pattern-matching/`**, **`control-flow/`**.
5. **`getting-started/`**, **`error-handling/`**, **`concurrency/`**.
6. **`tools/fmt-style.md`** — move file; update `fpas fmt` / `AGENTS.md` references.
7. Merge `11-stdlib.md` overview into `std/README.md`; drop duplicate hub if redundant.

### Phase 3 — Update references

Bulk-update paths in:

| Location | Estimate |
|----------|----------|
| Rust `///` and `//!` doc links | ~100+ |
| [`AGENTS.md`](../../AGENTS.md) | few |
| [`.cursor/rules/functional-pascal.mdc`](../../.cursor/rules/functional-pascal.mdc) | few |
| [`.github/instructions/functional-pascal.instructions.md`](../../.github/instructions/functional-pascal.instructions.md) | few |
| [`docs/pascal/`](../pascal/) internal cross-links | all pages |
| [`docs/future/`](../future/) | grep and fix |
| [`README.md`](../../README.md), [`examples/README.md`](../../examples/README.md) | few |
| [`docs/specs/grammar.ebnf`](../specs/grammar.ebnf) comments | few |

Use ripgrep before/after:

```sh
rg 'docs/pascal/0[0-9]-|docs/pascal/1[01]-|docs/pascal/fmt-style'
```

### Phase 4 — Remove legacy paths

1. Delete `01-overview.md` … `11-stdlib.md` and root `fmt-style.md`.
2. Confirm zero matches for old paths.
3. Update this file **Status** to *implemented* and move summary to [`README.md`](README.md) § Documentation.

---

## Verification

- [ ] Every old `##` section from numbered files has a new home (checklist in PR description).
- [ ] Root `docs/pascal/README.md` learning path is complete and ordered.
- [ ] Each area `README.md` lists all child pages.
- [ ] `rg 'docs/pascal/0[0-9]-'` returns no hits in the repo.
- [ ] Spot-check Rust links from `fpas-sema`, `fpas-compiler`, `fpas-vm`, `fpas-project`.
- [ ] No new `docs/rust/` paths introduced.

---

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Broken deep links (`#generics`) | Keep stable filenames; use explicit heading IDs only when needed |
| Merge conflicts during long migration | One area per PR; types → basics → program-structure order |
| Tutorial flow lost | “Start here” ordered list on root README |
| Contributor confusion | Single plan doc (this file) + area README hubs |

---

## Open decisions

1. **URL depth** — `language/types/records.md` vs flatter `types/records.md` at `docs/pascal/` root. Recommendation: **`language/`** prefix keeps program-structure and language separate.
2. **`11-stdlib.md`** — merge into `std/README.md` vs keep `program-structure/stdlib-overview.md`. Recommendation: **expand `std/README.md`** only.
3. **Stub period** — dual links in root README during migration vs big-bang PR. Recommendation: **one area per PR**, no long dual-maintenance.

---

## Related

- Current spec hub: [`docs/pascal/README.md`](../pascal/README.md)
- Stdlib reference pattern: [`docs/pascal/std/README.md`](../pascal/std/README.md)
- Contributor file-size rules: [`AGENTS.md`](../../AGENTS.md)
