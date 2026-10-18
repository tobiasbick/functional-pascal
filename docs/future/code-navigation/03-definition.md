# 03: N2 definition

Depends on [02](02-references.md). Add `fpas query definition` using the same
request, context, snapshot, diagnostic and JSON owners. No second CLI grammar.
The [overview](README.md) owns completion.

## Contract details

Return the actual defining identifier range as the single `matches` entry and
as `target.declaration`. Querying a declaration itself is a successful result.
Do not return a whole routine body or jump to the declaration of its result type.

An imported unit alias resolves to its alias declaration when the alias token is
selected; the original unit designator resolves to that unit. A member after an
alias resolves to the member's declaration. A callable variable resolves to the
variable/parameter, not a possible function assigned at runtime. Preserve these
same identities in N1. Named routine/callable actual labels and named record/enum
labels resolve to the source parameter or field they label.

For generic uses, return the original declared parameter/member selection span,
not a fabricated instantiated source. Visibility and multiple owning projects
follow the shared scope rules. A real standard-library/API source declaration is
valid; a builtin without source has an explicit `no_source_definition` miss.

## Work and acceptance

Reuse [definitions](../../../crates/fpas-language-service/src/navigation/service.rs)
and [resolution](../../../crates/fpas-language-service/src/navigation/resolve.rs).
Expose any needed target/diagnostic information in the shared typed query model,
without parsing hover strings or adding CLI-only name resolution. Avoid changing
`type_definitions`, which answers a different editor question.

Extend navigation tests for aliases, lexical shadows, callable values, methods,
constructors, generics and named labels; assert exact selected-name spans and
snapshot-bound Unicode ranges. Real CLI tests cover an imported source, a
self-declaration, private/out-of-scope/ambiguous targets, a builtin without source,
and malformed/degraded contexts. All outcomes use the N1 envelope and exit rules.

A complete source definition is one identity; never choose the first candidate
from an ambiguous array. No source edit or runtime evaluation occurs.

Update focused help, permanent docs, the wish list and plan after verification
and independent review. Existing N1, language-service/LSP and API-lookup tests
remain the owners of their unchanged contracts.
