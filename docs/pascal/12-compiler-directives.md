# 12. Compiler Directives

Compiler directives are special annotations embedded in source comments using the `{$...}` syntax.  They are processed by the **preprocessor** before the parser sees any tokens and allow conditional compilation, symbol management, and (in project mode) file inclusion.

## Syntax

```
{$KEYWORD argument}
```

The keyword and argument are separated by whitespace.  Keyword matching is **case-insensitive**.  The content between `$` and `}` is trimmed.

---

## Conditional Compilation

### `{$DEFINE name}`

Introduces a conditional symbol.  The name is normalised to upper-case.

```pascal
{$DEFINE DEBUG}
```

### `{$UNDEF name}`

Removes a previously defined symbol.

```pascal
{$UNDEF DEBUG}
```

### `{$IFDEF name}` / `{$IFNDEF name}`

Starts a conditionally compiled block.

```pascal
{$IFDEF DEBUG}
  WriteLn('Debug build');
{$ENDIF}

{$IFNDEF RELEASE}
  WriteLn('Not a release build');
{$ENDIF}
```

- `{$IFDEF name}` — the block is included when `name` is defined.
- `{$IFNDEF name}` — the block is included when `name` is **not** defined.

### `{$ELSE}`

Toggles the current conditional block.  May appear at most once between `{$IFDEF}` / `{$IFNDEF}` and `{$ENDIF}`.

```pascal
{$IFDEF RELEASE}
  const LogLevel = 0;
{$ELSE}
  const LogLevel = 3;
{$ENDIF}
```

### `{$ENDIF}`

Closes the innermost open `{$IFDEF}` or `{$IFNDEF}` block.

---

## Nesting

Conditional blocks can be nested to arbitrary depth:

```pascal
{$IFDEF PLATFORM_WIN}
  {$IFDEF DEBUG}
    WriteLn('Windows debug');
  {$ELSE}
    WriteLn('Windows release');
  {$ENDIF}
{$ENDIF}
```

An outer inactive block suppresses all inner `{$DEFINE}` and `{$UNDEF}` directives, even if their own conditions would otherwise be active.

---

## File Inclusion

### `{$I filename}` / `{$INCLUDE filename}`

Includes another source file inline at the current position.  **File inclusion is only available inside a multi-file project** (i.e. when building with `fpas build` and a `.fpproj` project file).

```pascal
{$I helpers.fpas}
{$INCLUDE shared/constants.fpas}
```

Using `{$INCLUDE}` in single-file mode (e.g., `fpas run file.fpas`) is a compile error.

---

## Predefined Symbols

No symbols are defined by default.  Symbols can be passed to the compiler from the outside (e.g., CLI flags) or defined inline with `{$DEFINE}`.

---

## Unknown Directives

Directive keywords not listed above (e.g., `{$R+}`, `{$O+}`) are **accepted but have no effect**.  A diagnostic is emitted in active code to warn the developer.  Unknown directives inside inactive branches are silently ignored.

---

## Error Conditions

| Situation | Diagnostic |
|-----------|-----------|
| `{$ELSE}` without a matching `{$IFDEF}` / `{$IFNDEF}` | Error F0010 |
| `{$ENDIF}` without a matching `{$IFDEF}` / `{$IFNDEF}` | Error F0011 |
| Unclosed `{$IFDEF}` / `{$IFNDEF}` at end of file | Error F0012 |
| Unknown directive keyword (in active branch) | Error F0013 |
| `{$INCLUDE}`  in single-file mode | Error F0014 |

---

## Complete Example

```pascal
program Greeting;

uses
  Std.Console;

{$DEFINE FORMAL}

begin
  {$IFDEF FORMAL}
    WriteLn('Good day, visitor.');
  {$ELSE}
    WriteLn('Hey!');
  {$ENDIF}
end.
```

Output (with `FORMAL` defined):

```
Good day, visitor.
```
