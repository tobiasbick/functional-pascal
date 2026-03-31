# Remove: `dict of K to V`

> Priority: 9 — last removal, evaluate after other simplifications.
> Decision pending — may be kept.

## What to remove (if proceeding)

- `dict of K to V` type constructor.
- Dict literal syntax: `['key': value]`, `[:]`.
- `for-in` over dict (key iteration).
- `Std.Dict` unit (`Length`, `ContainsKey`, `Keys`, `Values`, `Remove`,
  `Get`, `Merge`, `Map`, `Filter`).
- Keyword: `dict` (and `to` overload in type position).

## Arguments for removal

- No example uses dicts.
- Named lookups can use `array of record` with search functions.
- Removes 1 keyword, literal syntax, and the `Std.Dict` unit.

## Arguments for keeping

- Dicts are a fundamental data structure.
- Config, presets, and name-based lookups are clunky with arrays.
- Implementation cost is already paid.
- Removing and re-adding later creates churn.

## Recommendation

Evaluate after removals 1–8 are complete. If the language feels lean enough,
keep dicts. If further simplification is wanted, proceed with removal.
