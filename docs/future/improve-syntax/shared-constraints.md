# Shared constraints and regression checks

These checks supplement each work package's verification list and the
[development process](development-process.md). They are not a second backlog
or language specification. Apply the relevant boundaries when completing a
package; name any intentional behavior change in its approved contract.

## Planning boundaries

- Use the [current handbook](../../pascal/README.md) and
  [grammar](../../specs/grammar.ebnf) as the implemented baseline.
- Keep unresolved language choices explicit. Do not approve one indirectly
  through an implementation detail, dependency update or source migration.
- Prefer one canonical contract and established Pascal-family terminology.
  Do not add a parallel syntax, compatibility mode or speculative abstraction.
- Preserve evaluation count, ordering, effects and ownership unless the agreed
  scope changes them. Separate value transformation from caller mutation,
  expected error data from runtime failure, and completion from result retrieval.
- Keep checks at the owning public boundary and reuse shared implementations.
  Check parser recovery, diagnostics, runtime and tooling where they expose
  different observable risks.

## Regression boundaries

| Area | What to verify when affected |
| --- | --- |
| Arguments and calls | Parameter roles, named/positional rules, evaluation order and invalid argument diagnostics |
| Bindings and mutation | Read-only storage, explicit caller writes, aliases, callback effects and aggregate sharing |
| Names and imports | Qualified identity, collisions, visibility, callable labels and cached-unit consumers |
| Native operations | Published names, receiver shapes, constraints and errors agree with the current type handbooks |
| Patterns and case | Payload binding, closed-type coverage, constants, reachability and statement/expression consistency |
| Numeric and domain types | Type identity, inference, bounds, overflow and consistent checking/folding/runtime behavior |
| Blocks and literals | Nested ownership, lexical decoding, source spans, recovery and formatting round trips |
| Contracts and error handling | Allowed expressions, return timing, propagation, expected errors and preserved effects |
| Tasks and shared resources | Admission, ownership, cancellation, wait priorities, result consumption and cleanup |
| Projects and artifacts | Fresh and reused source-adjacent sidecars, interface identity and correct invalidation |
| Editor and CLI | Unsaved source, diagnostics, formatting, navigation, templates and public signatures |
| Migration | FPAS files, embedded fixtures, generators, generated API declarations, skills and handbook examples |

For each changed contract, identify positive, negative and boundary cases that
can distinguish correct behavior from a credible regression. Reuse suitable
existing tests. Use normal FPAS test programs and existing Rust test mechanisms;
do not add native test syntax or model-based acceptance gates.

## Current references

- [Language reference](../../pascal/agents/README.md)
- [Built-in type operations](../../pascal/language/types/README.md)
- [Dot calls](../../pascal/language/functions/fluent-calls.md)
- [Standard-library implementation touchpoints](../../pascal/std/README.md#shared-implementation-touchpoints)
- [FPAS test runner](../../pascal/std/testing/test.md)
