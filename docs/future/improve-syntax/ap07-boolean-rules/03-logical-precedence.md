# AP07.3: Logical precedence and mixing rules

Package: [AP07: Boolean rules](README.md)

## Scope

Implement the Q03 precedence table: comparisons above `not`, `not` above the
binary logical operators, same-operator logical chains only, and
non-associative comparisons.

## Work package boundaries

- Integer bit and shift operator removal belongs to AP07.4. The completed
  grammar has no infix shifts; integer bit operations use `Std.Bits`.
- The Q03 table reserves comparison-level precedence for `is`; AP20.3
  introduces its keyword, pattern syntax, and semantics. AP07.3 does not
  introduce the pattern test.

## Prerequisites

None. AP07.1 is recommended first so that migrated expressions are tested with
the final evaluation rule.

## Implementation

- Parser: restructure the precedence levels; reject mixed logical chains with
  a diagnostic showing the parenthesized form; keep comparison-chain rejection
  with the `A < B and B < C` hint.
- Formatter: mirror precedence and required parentheses.
- Debugger expression parsing follows the same table.

## Affected areas

- `crates/fpas-parser/src/parser/expr/precedence.rs`.
- `crates/fpas-fmt/src/emit/expr/`.
- `crates/fpas-debug/src/evaluation/` shared expression parsing and watch
  regressions.

## Migration

- Use the old parser's AST to find every logical expression whose grouping
  changes; add explicit parentheses that preserve the old meaning where it was
  intended, and record any expression that was a latent bug.
- Cover Rust-embedded fixtures and documentation examples.

## Documentation

- `docs/specs/grammar.ebnf` (expression productions) and
  `docs/pascal/language/basics/operators.md` with the complete table.

## Verification

- Parser tests for every precedence boundary, all mixed logical pairs,
  parentheses, `not` placement, and comparison chains.
- Formatter round trip; FPAS suite unchanged.

## Implementation result

- Logical chains and negation have a dedicated parser module. Mixed chains
  report the offending operator and show both parenthesized alternatives;
  comparison-chain recovery retains the following statement.
- The formatter preserves mixed logical groups, nested comparisons, and
  negated operands, including generated ASTs without explicit parentheses
  and expressions wrapped across lines.
- Debugger evaluation already uses the shared expression parser. No separate
  VM translation or precedence implementation is needed; watch regressions
  verify grouping and parse-error propagation through the protocol.
- An audit using the previous parser's AST covered repository FPAS sources,
  embedded fixtures, current documentation examples, and source generators.
  Existing logical expressions already preserve their grouping under the new
  table. No semantic source migration or latent regrouping bug was found.
