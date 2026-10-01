# Conventions and diagnostics

See the [steering document](README.md) for dependencies, decision gates, and
completion tracking. Proposed spellings below are not current-language examples.

## AP01: Pascal conventions

- [ ] Establish five to ten canonical examples for indentation, keywords,
  parenthesized calls, `:=`, and declarations. Label unresolved forms as drafts.
- [ ] Retain case-insensitivity and Pascal's routine, record, case, and block words.
- [ ] Keep `array of T` and use `of` for every type application, including
  user-defined generic types (AP24). Do not introduce `Array<T>`, `Option<T>`,
  or other angle-bracket type applications.
- [ ] Where Pascal or Delphi has an established spelling for a planned concept
  (subranges, distinct types, `var` parameters), use it unless a recorded
  decision says otherwise.
- [ ] Provide a Pascal-oriented reference style. A formatter change is not
  required for this step.

Acceptance: the reference examples state a consistent Pascal-oriented form
without presenting unresolved grammar as implemented behavior.

## AP02: Structured diagnostics

### Decided rules (Q01)

- Use stable numeric diagnostic codes with ranges per phase: `FP1xxx` for
  lexer errors, `FP2xxx` for parser errors, `FP3xxx` for semantic checks,
  `FP4xxx` for project/build diagnostics, and `FP5xxx` for runtime diagnostics.
- Show the codes in both human-readable and machine-readable output.
- Add `--diagnostics json` to `check`, `build`, `run`, and `test`, emitting
  one JSON object per line with code, severity, file, line, column, end position,
  message, expected, found, and hint.
- Add a diagnostics reference under `docs/pascal/tools/` during implementation,
  listing every code with a wrong and a corrected example.

### Tasks

- [ ] Audit the existing diagnostic schema and improve representative existing
  errors before inventing a parallel system.
- [ ] Provide a stable code, source position, short explanation, and expected
  versus actual value/type when applicable.
- [ ] Support both human-readable and machine-readable output.
- [ ] Include concrete correction hints without guessing business decisions.
- [ ] Diagnose comma-separated or grouped parameter declarations with the
  canonical semicolon-separated, individually typed form retained by AP08
  (Q06); cover both invalid forms and the valid form in regression tests.
- [ ] For every syntax change in this plan, recognize the superseded or
  commonly expected form (classic Pascal, Delphi, or other mainstream habits)
  and name the canonical replacement in the diagnostic.

Acceptance: tools can locate a representative error and determine its cause
without parsing explanatory prose, and superseded forms produce a hint with
the canonical spelling.

## AP03: Explicit closed-enum cases

### Decided rules

- A `case` over a closed enum (`Option`, `Result`, user enums) lists every
  variant explicitly; `else` is not allowed there. Open domains such as
  `integer` or `string` keep `else`.
- A guarded arm alone does not cover its variant. Ignoring payload fields
  (`_`, AP20) is distinct from ignoring a whole variant.
- Code that handles a single variant uses the `is` test from AP20 instead of a
  full `case`. The no-`else` rule ships together with or after that test.
- Variants that need no action are listed in an arm containing `null;` (AP13).

```pascal
case Msg of
  when TuiMsg.Key(const K):
    HandleKey(K);
  when TuiMsg.Tick, TuiMsg.Focus:
    null;
  // ... every other variant
end case;

if Msg is TuiMsg.Key(const Key) then
  HandleKey(Key);
end if;
```

### Tasks

- [ ] Inventory `case ... else` sites over closed enums in `lib/`, `apps/`,
  `examples/`, and `tests/`. Large enums at planning time: `TuiElement` (43
  variants), `TuiMsg` (37), `TuiStyleRole` (34), `KeyKind` (29).
- [ ] Migrate each site to a full variant list or an `is` test.
- [ ] Diagnose `else` on a closed enum with the list of variants it replaced.
- [ ] Add a variant and verify that every incomplete case reports the missing
  variant while complete cases remain accepted.

Acceptance: enum extension exposes omitted cases; field wildcards do not
silently bypass explicit top-level variant coverage; single-variant handling
uses the `is` test.

## AP04: Discarded function values

### Decided rules

- A function result in statement position must be consumed. Procedures remain
  the ordinary standalone calls.
- `discard Expression;` is the explicit form for ignoring a function result.
  `discard` becomes a reserved keyword.
- `Result` values may be discarded only with `discard`; an unused `Result`
  without it is an error.
- Task handles cannot be discarded. `discard go Worker();` is an error whose
  diagnostic points to the existing statement `go Worker();`; task lifetime
  stays with `go` statements and AP26.

```pascal
Fs.Delete(TempPath);           // error: unused Result value
discard Fs.Delete(TempPath);   // valid: deliberately ignored
discard go Worker();           // error: use the statement 'go Worker();'
go Worker();                   // valid: fire-and-forget task
```

### Tasks

- [ ] Specify `discard` for a procedure call (no value) as an error.
- [ ] Diagnose accidental result loss with a hint showing `discard`, and for
  `Result` values also `case` or `try`.
- [ ] Migrate existing statement-position function calls, including the final
  function call of a postfix chain.
- [ ] Test ordinary values, `Result`, `Option`, task handles, procedures, and
  postfix chains.

Acceptance: explicit discard is usable for ordinary values and `Result`, is
visible in source, and cannot be used to drop a task handle.
