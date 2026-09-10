# Strict ISO/IEC 14977 grammar

> **Decision status: OPEN.** The current preference leans toward converting
> `docs/specs/grammar.ebnf` to strict ISO/IEC 14977, but that direction has not
> been approved as the final design.

## Why this decision exists

The current grammar labels its notation as ISO 14977 EBNF, but it uses a
project-local mixture of EBNF conventions. It remains readable to a person,
yet a strict ISO 14977 parser cannot consume it reliably.

The mismatch includes:

- implicit concatenation by adjacency instead of the ISO comma operator;
- underscores in production names;
- `..` character ranges, which ISO 14977 does not define;
- prose comments used as placeholders inside productions;
- a comment example nested inside another comment;
- no automated syntax validation for the grammar itself.

The grammar's authority also needs an explicit boundary. It currently combines
lexical rules, parser productions, and explanatory semantic restrictions. That
makes it unclear whether it describes valid FPAS source, every token sequence
from which the recovering parser can construct an AST, or the canonical output
of `fpas fmt`.

## Proposed contract

If strict ISO 14977 is selected, the grammar should describe the syntax of
valid FPAS programs.

- Parser recovery is an implementation detail. A parser may construct a
  partial AST for invalid input while still reporting diagnostics; those
  recovery extensions do not belong in the grammar.
- Formatter output is a canonical presentation of valid syntax, not the set of
  all valid syntax. Formatter-only choices remain in
  `docs/pascal/tools/fmt-style.md`.
- Type rules, visibility rules, exhaustiveness, and other semantic constraints
  remain in the language documentation. The grammar may link to those rules
  but should not pretend to express them as syntax.

This contract is part of the open decision. Choosing strict notation without
choosing what the notation means would only replace one ambiguity with a more
carefully punctuated ambiguity.

## What strict ISO conformance would require

### Explicit concatenation

Every adjacent syntactic term would receive the ISO concatenation operator.

Current form:

```ebnf
identifier = letter { letter | digit } ;
```

Strict form:

```ebnf
identifier = letter, { letter | digit } ;
```

This is a mechanical but broad rewrite across almost every production.

### ISO-compatible meta-identifiers

Production names such as `integer_literal`, `uses_clause`, and
`record_update` would need an ISO-compatible representation, likely
`integer literal`, `uses clause`, and `record update`.

That rename affects every production definition, every internal reference,
and every production reference under `docs/pascal/`. The generated training
dataset must then be regenerated from the corrected documentation rather than
edited by hand.

### Character sets and ranges

The current notation uses ranges such as:

```ebnf
letter = 'A' .. 'Z' | 'a' .. 'z' | '_' ;
```

ISO 14977 has no `..` range operator. A conversion must choose between:

- enumerating every terminal character; or
- using ISO special sequences such as
  `? ASCII uppercase or lowercase letter ?`.

Enumeration is verbose but mechanically explicit. Special sequences keep the
grammar compact, but ISO defines only their delimiters, not their meaning. Any
validator or consumer would therefore need a documented FPAS interpretation
for each permitted special sequence.

The same decision applies to string contents, line-comment contents, Unicode
source characters, and line endings.

### Comments and prose placeholders

ISO comments cannot be nested. The notation legend must not place a literal
`(* ... *)` example inside another comment.

Productions such as the current conceptual `string_char` rule must use a
well-defined special sequence or explicit character grammar instead of a prose
comment standing in for syntax.

### Documentation references

All production references in `docs/pascal/` must use the final ISO
meta-identifiers. This includes the references for declarations, type forms,
control flow, error handling, concurrency, records, and pattern matching.

The conversion should be atomic: do not leave language pages referring to the
old underscore names while the grammar already defines the new names, or vice
versa.

### Training data

The FPAS training dataset ingests `docs/pascal/`. After the documentation
references change, regenerate `training/fpas/data/*.jsonl` and
`training/fpas/manifests/dataset-v1.json` through the existing generator.

Dataset validation should reject obsolete production names so an old grammar
vocabulary cannot silently return as positive training material.

## Automated drift guard

Strict conversion is worthwhile only if conformance remains checked. The
repository should gain an automated grammar validation step that:

1. parses `docs/specs/grammar.ebnf` as ISO/IEC 14977;
2. rejects undefined and duplicate meta-identifiers;
3. checks that the grammar's keyword inventory matches `fpas-lexer`;
4. checks that production references in `docs/pascal/` exist;
5. recognizes only an explicit allowlist of FPAS special sequences;
6. reports a precise file location and corrective hint;
7. has regression tests for malformed grammar, keyword drift, broken
   references, and unsupported special sequences.

The guard should run through the normal workspace test suite. It must not rely
on a globally installed parser or a network service.

The grammar does not need to generate the production parser. Parser generation
would be a separate architectural decision with much larger consequences. The
initial guard only needs to make the documented grammar structurally valid and
detect known forms of drift.

## Risks and costs

- The grammar becomes more verbose, especially because of explicit
  concatenation and character definitions.
- A large mechanical conversion can accidentally alter language meaning even
  when no language change is intended.
- ISO-valid special sequences may still be meaningless to tooling unless FPAS
  defines their interpretation.
- Renaming every production creates coordinated churn in language pages and
  generated training data.
- A validator that checks only punctuation can create false confidence; the
  lexer and parser still need behavior-level regression tests.

These risks argue for a conversion with semantic before-and-after checks, not
for a blind search-and-replace pass.

## Alternative: name the existing dialect

The smaller alternative is to remove the ISO claim and define the notation as
the FPAS EBNF dialect. Its adjacency, ranges, underscore names, and prose
extensions would then be documented explicitly, followed by a validator for
that dialect.

This would require less churn and remain easier to read, but it would preserve
a project-specific grammar format and require maintaining a custom parser or
validator indefinitely.

## Suggested implementation sequence if ISO is chosen

1. Approve the proposed contract: valid FPAS syntax, excluding recovery and
   formatter-only rules.
2. Choose the policy for character enumeration and special sequences.
3. Implement the ISO grammar parser and structural validator with tests.
4. Convert the grammar without intentionally changing FPAS syntax.
5. Compare keyword inventory and all existing productions before and after the
   conversion.
6. Update every production reference in `docs/pascal/`.
7. Regenerate and validate the FPAS training dataset.
8. Run formatter, build, workspace tests, FPAS tests, and link checks.
9. Record the decision and remove any temporary compatibility checks for old
   production names.

## Acceptance criteria

The work is complete only when:

- the decision is no longer marked open;
- the grammar's stated notation and actual notation agree;
- the grammar explicitly describes valid FPAS syntax;
- a strict parser accepts the complete grammar;
- every referenced production is defined exactly once;
- the grammar and lexer expose the same case-insensitive keyword set;
- all special sequences have documented FPAS meanings;
- all language-page references use the final production names;
- generated training data contains no obsolete grammar vocabulary;
- existing parser, compiler, formatter, application, and FPAS regression tests
  remain green.

## Open questions

1. Should character classes be enumerated, expressed as allowlisted special
   sequences, or use a deliberate mixture of both?
2. Which component should own the ISO parser and drift checks?
3. Should documentation refer to spaced ISO meta-identifiers verbatim, or use
   stable anchors generated alongside them?
4. Is strict syntactic validation sufficient for the first version, or must
   the guard also compare selected parser behaviors immediately?
5. Should the final decision be recorded as an ADR in addition to this future
   plan?
