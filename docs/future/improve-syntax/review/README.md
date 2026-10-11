# Syntax implementation review plan

The plan is empty. No review is active and no repair items are open.
Previously reviewed AP packages and work packages remain in
[reviewed-scope.md](reviewed-scope.md). Completed findings and their repair
evidence remain in Git history.

The [central plan](../README.md) owns package implementation status. This
directory records review scope, findings and verification; it does not change
package completion or authorize new language rules.

## Files for the next review

Keep these files and replace their empty-state text when starting a review.
Use English throughout and keep each finding in its owning detail file.

| File | Contents |
| --- | --- |
| [README.md](README.md) | Fixed baseline and target revisions, selected AP work packages, prioritized finding index, repair order and current status |
| [reviewed-scope.md](reviewed-scope.md) | Persistent record of completed review coverage; append the next reviewed scope after closure |
| [compiler-and-semantics.md](compiler-and-semantics.md) | Spec findings in parsing, semantic analysis, compilation, linking and runtime behavior |
| [tooling-and-tests.md](tooling-and-tests.md) | Spec findings in editor services, CLI, formatter, tests, migrations and current documentation |
| [rust-standards.md](rust-standards.md) | Standards findings in module ownership, structure, Rust code and test quality |
| [verification.md](verification.md) | Commands actually run, results, regression evidence, second-review results and coverage limits |

## Starting a review

1. Record full baseline and target commit IDs before inspecting changes. State
   the selected AP work packages, implemented behavior and excluded areas.
   Recheck the central plan and the persistent coverage record; prior review
   coverage does not exempt later changes from review.
2. Review **Spec** against the agreed AP requirements and implemented handbook
   rules. Review **Standards** against [AGENTS.md](../../../../AGENTS.md),
   [development-process.md](../development-process.md) and applicable skills.
   Keep the two results separate and avoid counting one defect twice.
3. Confirm findings with a requirement, concrete evidence, actual versus
   expected behavior, an owning code path and bounded acceptance criteria.
   Keep unconfirmed suspicions out of the repair index.
4. Add an index with the columns `Status`, `ID`, `Priority`, `AP/work package`,
   `Finding` and `Details`. Link each entry to its detail heading. Use `Snn`
   for compiler/spec findings, `Tnn` for tooling/spec findings and `Rnn` for
   independent standards findings. Order repairs by impact and dependencies.

Use P1 for high-impact correctness defects, P2 for other reproducible contract
violations and P3 for limited presentation or maintenance defects. Explain the
impact rather than assigning priority from file size or suspicion alone.

## Repairing and closing findings

For each finding, demonstrate the failure at the owning public boundary,
implement the bounded repair, run applicable checks, and review the result
again. Then update its detail, index and verification evidence and commit that
item before starting the next one.

Check docs and tests for every implementation or behavior change. Current
handbook rules must stay aligned with implemented behavior. A proposed syntax,
semantic or user-facing language-spec change requires explicit user agreement
before implementation.

At closure, append the reviewed AP work packages, fixed revision range and
coverage limits to `reviewed-scope.md`. Keep historical coverage when clearing
the active plan and detail files for the following review. Review completion
alone is not merge approval or an exhaustive feature-combination claim.
