# Text, encodings, and regular expressions

Strings are immutable UTF-16 references. Length and indexes count code units,
and indexing returns char. Use System.Text.Rune for Unicode scalars and the
appropriate .NET globalization/text APIs for graphemes and culture-sensitive work.

Use Encoding for text/byte conversion and System.Text.RegularExpressions.Regex
for matching/replacement. Select ordinary string/array overloads supported by the
compiler; byref-like Span signatures are outside the current scope. Pattern
syntax, options, timeout, and escaping follow the actual .NET regex API.

## Example

```pascal
program TextAndRegex;

uses
  System as Sys, System.Console as Console, System.Text as Text,
  System.Text.RegularExpressions as Regular;

begin
  const Bytes: array of byte := Text.Encoding.UTF8.GetBytes('Aé');
  const Decoded: string := Text.Encoding.UTF8.GetString(Bytes);
  const Replaced: string := Regular.Regex.Replace('room12', '[0-9]+', '#');
  if Bytes.Length <> 3 or Decoded <> 'Aé' or Replaced <> 'room#' then
    raise new Sys.InvalidOperationException('Incorrect encoding or regex result');
  end if;

  Console.WriteLine(Replaced);
  Console.WriteLine(Bytes.Length);
end;
end program;
```

## See also

- [Literals](../language/basics/lexical-rules.md)
- [Primitive strings and characters](../language/basics/primitive-types.md)
- [.NET text reference](https://learn.microsoft.com/en-us/dotnet/api/system.text)
- [.NET regex reference](https://learn.microsoft.com/en-us/dotnet/api/system.text.regularexpressions.regex)
