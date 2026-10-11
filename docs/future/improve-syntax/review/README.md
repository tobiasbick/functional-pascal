# Syntax implementation review and repair plan

This review checks implementation delivered after the
[earlier completed review](reviewed-scope.md). It records **six confirmed
findings: one P1, four P2 and one P3**. S01/S02/S03/T01/T02 are repaired and verified;
T03 remains open. The repaired P1 defect admitted incorrectly typed
Result payloads in decision expressions.

The [central plan](../README.md) remains the only package-completion index.
Checkboxes here track repairs, not a second package backlog. The earlier
review's 22 resolved findings are not reopened. Repairs implement the existing
language and editor rules; no language change is included.

## Fixed scope

- Target: `34397373799a5897dfcfc09e754d16fcad3c7386`.
- Baseline: `5b04c39697f2d57e6390648194ae294ddff614bb`, the completed review reset.
- Comparison: `git diff 5b04c3969...343973737` and
  `git log 5b04c3969..343973737 --oneline`.
- The fixed scope identifies the reviewed implementation. Repair checkboxes
  describe delivery on the current integration branch.

Scope follows the implemented packages absent from the earlier coverage record,
including the four expressly named in the former review placeholder.

| Package | Reviewed implementation and interactions |
| --- | --- |
| AP03 | Closed-enum coverage, guards, scalar exceptions and case expressions |
| AP15 | Optional-handler fields, removed event owners, existing consumer coverage |
| AP18 | Subrange bounds, conversions, membership, diagnostics and generic/record use |
| AP19 | Distinct identity, conversions, comparisons, constraints and case labels |
| AP21 | If/case expression typing, lowering, constants, pattern scope and editor binding |
| AP22 | Allowed inference, constructor/conversion evidence and editor integration |
| AP23 | Contracts, exits, captures, indirect/generic calls and unit diagnostics |
| AP24 | Generic records/enums, joint inference, recursion, callable fields and symbols |

Three subagents reviewed generics/inference, subranges/contracts, and
standards/tooling. The coordinating review inspected AP03, AP15, AP19 and AP21,
reproduced findings and ran shared checks. Current package specifications and
handbook rules were the requirements. Pending packages AP12, AP25, AP26 and
AP27 were not judged as missing implementations.

## Spec findings

| Done | Finding | Priority | Confirmed problem | Repair and acceptance |
| --- | --- | --- | --- | --- |
| [x] | S01 | P1 | If/case joins erase Result payload evidence; invalid values reach runtime | [Preserve branch types](compiler-and-semantics.md#s01--p1-decision-expressions-lose-complementary-result-type-information) |
| [x] | S02 | P2 | Joint record/enum inference omits generic callbacks inside containers | [Complete contextual inference](compiler-and-semantics.md#s02--p2-joint-constructor-inference-stops-at-nested-generic-callables) |
| [x] | S03 | P2 | Scalar conversions accept type names as values and reach FP9001 | [Validate value positions](compiler-and-semantics.md#s03--p2-scalar-conversions-accept-type-declarations-as-value-arguments) |
| [x] | T01 | P2 | Generic data type parameters are absent from editor symbols | [Extract parameter bindings](tooling-and-tests.md#t01--p2-generic-data-type-parameters-are-missing-from-editor-symbols) |
| [x] | T02 | P2 | A case-expression binder resolves to an outer constant | [Respect expression scopes](tooling-and-tests.md#t02--p2-case-expression-bindings-resolve-to-a-shadowed-outer-constant) |
| [ ] | T03 | P3 | Generic callable-field signature help shows template types | [Specialize signatures](tooling-and-tests.md#t03--p3-generic-callable-field-signature-help-remains-unspecialized) |

Each item records its requirement, input/observed failure, production owner,
repair scope and positive/negative acceptance cases. Unconfirmed suspicions are
excluded. No additional contract bypass or independent documentation defect
was established.

## Standards result

The [Rust standards review](rust-standards.md) found **zero separate actionable
structural/style violations**. Semantic/editor defects stay on the Spec axis
and are not counted again as standards findings.

## Repair order and exit gates

1. Fix S01 first: accepted programs currently violate their declared type.
2. Repair S02 and S03 in their semantic owners; no new language decision is needed.
3. Repair T01/T02 at symbol extraction and scope ownership, then T03 at shared
   member specialization. Avoid independent query-specific workarounds.
4. Check affected handbook pages against the repaired behavior. Existing rules
   remain binding unless a separate language change is explicitly agreed.
5. Demonstrate each regression failing before repair and passing afterward;
   extend behavior-owning tests with distinct positive, negative and edge cases.
6. Run the [required checks](../development-process.md#required-checks), affected
   editor tests and each finding's acceptance criteria before closing items.

This is not a merge approval. Existing-suite results and the six targeted
failures are separate evidence; a green suite does not close these findings.
See [verification.md](verification.md) for actual commands, outcomes and limits.
