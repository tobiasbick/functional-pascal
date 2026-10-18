# 04: N3 checked type at a source position

Depends on [02](02-references.md) and [03](03-definition.md). Add
`fpas query type` within the shared request/scope/result contract. The
[overview](README.md) owns completion. This is a static query, not debugger
expression evaluation or a jump to a type declaration.

## Selected construct

For a position in source code, select the innermost real typed expression whose
half-open source range contains that position. Smaller nested expressions win;
syntax-tree depth breaks equal-span ties. A position in an argument selects
that argument's inner expression; an operator in `A + B` selects the binary
expression. A routine name as the callee of a call selects the call result;
a bare routine value selects its callable type.

Postfix paths need their actual typed prefixes: on `Items` in
`Items[0].Name.Length()` report the array type; on `Name` report its field type;
on `Length`/its call suffix report the call result. An index expression still
selects its own type. Use the checker’s path/postfix metadata for those suffixes,
not the type of the whole chain for every token.

For a selected value-declaration name, parameter, loop/pattern binding or named
actual/constructor-field label, return its checked receiving/value type in that
context. Named labels take precedence over the containing call expression.
Declarations of types, unit/import names and procedures used as statements do not
become fictitious value expressions. Their value-type query is `not_found`;
N2 remains available for navigation. Comment text never selects a containing expression. Any position inside a
string literal, including its delimiters/content, selects the literal’s string
type rather than a name written inside it or an enclosing call. Whitespace between typed operands can select their containing expression,
while whitespace with no containing typed expression is a miss.

A parse error, `Ty::Error`, missing checked metadata or unresolved generic/member
selection must never produce a successful guessed type. Preserve diagnostics and
report incomplete coverage. A clean selection with no applicable typed construct
is `not_found`.

## Type result

`type` contains `display`, a stable `kind` derived from the checked `Ty`, the
selected source `location`, and an optional nominal `declaration` location.
`matches` stays empty. Categories follow semantic types (integer, real, boolean,
string, array, dict, record, enum, option, result, callable, task, channel,
subrange, distinct and generic parameter); centralize the exhaustive mapping.
Use canonical compiler type rendering for `display`, preserving nominal owners,
generic substitutions and callable parameter modes/labels. A resolved semantic
type that has no source declaration has `declaration: null`, not a fake range.

Do not serialize the compiler's internal recursive type graph, introduce a second
user-facing type syntax, parse symbol/hover text, or erase a distinct type to its
representation merely to obtain an answer.

## Actual implementation owners

- [Semantic document analysis](../../../crates/fpas-language-service/src/analysis/document.rs)
  exposes `SemanticAnalysis::metadata()` for the retained source analysis.
- [Compiler metadata](../../../crates/fpas-sema/src/check/context.rs) already holds
  identity-keyed expression types and source-keyed binding/pattern/path/postfix
  types. Identity keys are valid only for their original immutable AST allocations.
- [Syntax selection](../../../crates/fpas-language-service/src/navigation/selection.rs)
  has source traversal/range infrastructure; use a focused typed selection owner
  rather than making the CLI walk the AST or pretending every syntax selection
  is a checked expression.
- [Inferred bindings](../../../crates/fpas-language-service/src/symbols/inferred.rs)
  already supply checked types to editor symbols. This is not a complete expression
  type API and must not become a hover-string parsing workaround.

Expose a protocol-independent typed result in the language service. If traversal
needs additional source-keyed checked type information, retain it at the shared
checker/analysis boundary with the originating snapshot. Do not reparse or clone
an AST and then look up another allocation's identity-keyed metadata. Cover project
units as well as loose programs; preserve the semantic checker as the authority.

## Acceptance

Use direct language-service tests for literals, unary/binary expressions, nested
calls, postfix receiver/index/field/call selection, inferred locals, declaration
names, closures/callable values, named labels, generics, nominal/distinct/subrange
types, Option/Result, patterns, arrays/dicts and task/channel types. Assert checked
types and exact source selections, including repeated equal-looking expressions
and multi-unit analysis that could lose AST identity.

Real CLI tests cover a representative nested expression, an inferred/nominal
binding, a source dependency, parse/type failures, an unsupported/non-value
selection, Unicode ranges and shared lifecycle/byte limits. Ensure a failing
query never runs initializer/default/call code or creates source/build artifacts.
Keep N1/N2 and editor hover/type-definition behavior covered at their own owners.

After independent review and applicable workspace checks, publish permanent
help/docs and reference/skill workflow notes, mark N3 implemented and remove the
completed temporary plan as described in the overview. N4 rename needs its own
later scope and approval; it is not an automatic continuation of N3.
