# AP32.2: Current-language examples and terminology

Package: [AP32](README.md). Status: pending documentation correction.

## Scope and prerequisites

Correct the package's remaining documentation findings against implemented
AP03/AP13/AP21/AP26 behavior. Do not change semantics or wait for proposed
replacement rules. Treat `discard` as a keyword distinct from dropping use of
a binding; distinguish a task without a scope owner from a spawn without a
retained handle.

## Affected areas and documentation

- `docs/specs/grammar.ebnf`: remove FPAS-looking illegal method desugaring.
- `docs/pascal/language/types/record-methods.md`: explain implicit Self in prose.
- `docs/pascal/language/control-flow/case-of-intro.md`: align Boolean fallback
  guidance with `pattern-matching/exhaustiveness.md`.
- `docs/pascal/language/concurrency/task-handles.md`, `task-scopes.md`, and
  `docs/pascal/std/concurrency/task.md`: precise handle and ownership terms.
- Pattern/control-flow examples and `docs/pascal/agent-reference.md`: prefer
  direct body statements and `null` when an additional scope is unnecessary.

## Migration and verification

Documentation only. Preserve explicit nested blocks that establish a useful
scope. Check changed complete examples or containing fixtures with `bin/fpas`,
validate relative links and grammar comments, and run `git diff --check`.
Do not advertise `discard StartTaskInGroup(...)` as supported.
