# Future: Compiler Directives

> Deferred from v1. Planned for a future version.

Delphi and FreePascal support compiler directives inside comments using the `$` prefix. These control conditional compilation, file includes, and compiler settings.

## Examples (Delphi/FreePascal Style)

```pascal
{$IFDEF DEBUG}
  Std.Console.WriteLn('Debug mode');
{$ENDIF}

{$I config.inc}        { include file }

{$R+}                  { range checking on }
{$OPTIMIZATION ON}
```

## v1 Behavior

In v1, `{$...}` is treated as a regular brace comment and ignored. The lexer does not special-case the `$` prefix.

## Future Considerations

- Conditional compilation: `{$IFDEF}`, `{$IFNDEF}`, `{$ELSE}`, `{$ENDIF}`
- Include files: `{$I filename}` or `{$INCLUDE filename}`
- Compiler switches: `{$R+}` range checking, `{$O+}` optimization
