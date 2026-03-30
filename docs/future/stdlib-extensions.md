# Future: Standard Library Extensions

> Remaining planned extensions. Functions already implemented are documented in [docs/pascal/std/](../pascal/std/README.md).

---

## ~~Std.Str — String Indexing~~ — **implemented**

### Motivation

Accessing a single character in a string currently requires `CharAt(S, I)` or `Substring(S, I, 1)`. Direct index syntax would be more natural.

### ~~Proposed Syntax~~ Syntax

```pascal
var S: string := 'Hello';
var C: char := S[0];            { 'H' — 0-based character index }
```

Index access on `string` yields `char`. Out-of-bounds is a runtime error.

See [02-basics.md](../pascal/02-basics.md) for documentation.

---

## ~~Std.Dict — Functional Transformations~~ — **implemented**

### Motivation

`Get` and `Merge` are implemented. Still missing: higher-order `Map` and `Filter` on dictionaries — needed for transforming config data or filtering entries.

### ~~Proposed Additions~~ Additions

| Kind | Signature | Description |
|------|-----------|-------------|
| function | `Map(D: dict of K to V; F: function(V: V): V2): dict of K to V2` | Transform all values |
| function | `Filter(D: dict of K to V; F: function(K: K; V: V): boolean): dict of K to V` | Keep matching entries |

See [std/dict.md](../pascal/std/dict.md) for documentation and examples.

---

## ~~Std.Console — Timer and Timeout Events~~ — **implemented**

### Motivation

Interactive programs need idle processing, cursor blinking, and periodic updates. `ReadEvent()` currently blocks indefinitely — there is no way to specify a timeout or receive periodic timer events.

### ~~Proposed Additions~~ Additions

| Kind | Signature | Description |
|------|-----------|-------------|
| function | `ReadEventTimeout(Milliseconds: integer): Option of Event` | Wait up to `Milliseconds` ms; return `None` on timeout |
| function | `PollEvent(): Option of Event` | Non-blocking; return `None` if no event pending |

**Note:** `EnableRawMode()` must be called before using either function.

See [std/console.md](../pascal/std/console.md) for documentation and examples.

---

## ~~For-In over Dict~~ — **implemented**

### Motivation

Dictionaries (`dict of K to V`) cannot be iterated with `for-in`. The only way to iterate is `Keys(D)` then index lookup, which is indirect and less efficient.

### Proposed Syntax

```pascal
uses Std.Dict;

var Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];

for Key: string in Ages do
  WriteLn(Key + ': ' + IntToStr(Ages[Key]));
```

Iteration yields keys in insertion order (matching `Keys()` behavior). A key-value form could also be considered:

```pascal
for Key: string, Value: integer in Ages do
  WriteLn(Key + ': ' + IntToStr(Value));
```

---

## Docs and Specs to Extend (when implemented)

- ~~[02-basics.md](../pascal/02-basics.md): string index access `S[I]`~~ — done
- ~~[03-control-flow.md](../pascal/03-control-flow.md): `for-in` over `dict`~~ — done
- ~~[std/dict.md](../pascal/std/dict.md): `Map`, `Filter`~~ — done
- ~~[std/console.md](../pascal/std/console.md): `ReadEventTimeout`, `PollEvent`~~ — done
- ~~[grammar.ebnf](../specs/grammar.ebnf): `ForInDict` production~~ — done
- ~~[grammar.ebnf](../specs/grammar.ebnf): `StringIndexExpr` production~~ — covered by existing `designator` rule
