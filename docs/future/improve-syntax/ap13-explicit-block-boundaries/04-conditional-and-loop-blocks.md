# AP13.4: Conditional, loop, and scoping blocks

Package: [AP13: Explicit block boundaries](README.md)

## Scope

Introduce `if ... elsif ... else ... end if;`, `for ... end for;`, and
`while ... end while;` with statement-list bodies, `null;` for empty bodies,
and plain `begin ... end;` compound statements with their own scope (Q08).
`repeat ... until C;` stays.

## Prerequisites

- AP13.1 (`elsif`, `null` reserved).
- AP13.2 (statement terminators).

## Implementation

- Parser: branch and loop bodies are statement lists; `elsif` continues an
  `if`; `else if` starts a nested `if` with its own `end if;`. A missing
  `end if;` after `else if` suggests `elsif`.
- Reject a plain `end;` used to close a control structure.
- `null;` statement; an empty statement list is an error.
- Sema: each branch and loop body is a scope; plain blocks keep a nested scope
  whose declarations are not visible outside.
- Formatter: indentation of branch and loop bodies.

## Affected areas

- `crates/fpas-parser/src/parser/stmt/branching.rs`, `stmt/loops.rs`.
- `crates/fpas-sema/src/check/stmt/` (scopes).
- `crates/fpas-fmt/src/emit/stmt/`; editor snippets and indentation.

## Migration

Rewrite every `if`, `for`, and `while` in all repository consumers; nested
`else if` chains become `elsif` where they continued the chain. Preserve branch
ownership exactly.

## Documentation

- `docs/specs/grammar.ebnf` (`if_stmt`, `for_stmt`, `for_in_stmt`,
  `while_stmt`, `block`), `docs/pascal/language/control-flow/`, `fmt-style.md`.

## Verification

- Nested conditionals, `elsif` chains, loops inside branches, empty bodies with
  `null;`, plain blocks inside branches and loops, local declaration
  visibility and rejection outside the block.
- Execution tests for condition evaluation order and returns through `elsif`.
