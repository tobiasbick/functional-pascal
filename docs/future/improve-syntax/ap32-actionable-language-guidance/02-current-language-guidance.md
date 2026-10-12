# AP32.2: Current-language examples and terminology

Package: [AP32](README.md). Status: complete.

## Result

Guidance follows implemented AP03/AP13/AP21/AP26 behavior. Instance method
examples use value dot notation and explain implicit `Self` without a
type-qualified instance call. Boolean case-expression guidance includes the
valid `else` fallback and distinguishes statement coverage.

Task guidance distinguishes a task without a lexical scope owner from a spawn
without a retained handle. Group-owned handles are retained in bindings;
leaving a binding unused does not detach its child, and the `discard` keyword
still rejects task-containing values. Branch and loop examples use direct
statement lists while retaining explicit blocks that teach an additional scope.

## Affected areas and documentation

- `docs/specs/grammar.ebnf` and
  `docs/pascal/language/types/record-methods.md` explain implicit `Self`.
  Grammar productions are unchanged.
- `docs/pascal/language/control-flow/case-of-intro.md` and
  `docs/pascal/agent-reference.md` describe Boolean expression fallback.
- `docs/pascal/language/concurrency/task-handles.md`, `task-scopes.md`,
  `docs/pascal/std/concurrency/task.md`, and the agent reference distinguish
  handle retention, lexical ownership, and `discard`.
- `docs/pascal/language/pattern-matching/guards.md`,
  `docs/pascal/language/control-flow/for-in.md`, `break-continue.md`, and
  the cancellation example in `docs/pascal/std/concurrency/task.md` use direct
  body statements. The agent reference retains its nested-block scope example.

## Verification coverage

Documentation only, with no source-syntax migration. Changed Pascal examples
are checked in complete containing programs. These cover bound methods,
Boolean fallback, guards, direct loop bodies, and cancellation. Task checks
exercise a retained handle outside a scope and an unused group handle observed
through group close; `discard StartTaskInGroup(...)` remains rejected.

`crates/fpas-cli/src/main_tests/examples/agent_reference.rs` checks and runs
all complete agent-reference fences. Relative documentation references resolve,
and the grammar change is confined to explanatory comments.

## Open follow-ups

None within this work package's scope.
