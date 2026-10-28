# `Std.Cli`

Schema-based parsing of program arguments with typed values, recoverable errors,
and generated help. This source standard-library unit does not read process
arguments, print text, or exit the program.

## Importing and names

Import `uses Std.Cli as Cli;` and access public functions and types through
`Cli`. [`Std.Os.Args`](args.md) supplies process arguments separately. The parser
takes an array of argument strings, excluding the program name.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `Parse(Arguments: array of string; Definition: Schema): Result of (Outcome, CliError)` | validates the schema and parses arguments or returns a help outcome |
| function | `Help(Definition: Schema): Result of (string, CliError)` | validates the schema and returns help text ending in LF |
| type | `Kind` | `Text`, `Int`, `Float`, or `Bool` |
| type | `Value` | `Text(Value: string)`, `Int(Value: integer)`, `Float(Value: real)`, or `Bool(Value: boolean)` |
| type | `OptionSpec` | definition of one named option |
| type | `PositionalSpec` | definition of one positional value |
| type | `Schema` | program description and ordered option/positional definitions |
| type | `Outcome` | `Parsed(Values: dict of string to Value)` or `Help(Text: string)` |
| type | `CliError` | recoverable schema, argument, value, and limit failures |

The facade aliases the types declared in `Std.Cli.Types`. In particular,
`Cli.CliError` is the same nominal error family as `Std.Cli.Types.CliError`;
the alias does not create another error type. Applications import the public
`Std.Cli` facade.

## Schema fields

`OptionSpec` has these fields:

| Field | Type | Default / meaning |
|-------|------|-------------------|
| `Name` | `string` | required canonical long name and result key |
| `Short` | `Option of string` | `None`; optional one-letter alias |
| `Kind` | `Cli.Kind` | `Cli.Kind.Text` |
| `Default` | `Option of Cli.Value` | `None`; explicit default of the matching kind |
| `Required` | `boolean` | `false`; require an explicit occurrence |
| `Help` | `string` | `''`; description for generated help |

`PositionalSpec` has `Name`, `Kind`, `Default`, `Required`, and `Help` with
the same types. Its `Required` default is **true**, so set it to `false` for
an optional position. It has no `Short` field.

`Schema` has required `ProgramName: string`, `Options: array of Cli.OptionSpec`,
and `Positionals: array of Cli.PositionalSpec` fields, plus
`Description: string := ''`. Supply both arrays explicitly, including `[]`
when empty. `ProgramName` is used only in help; it is not consumed from the
argument array.

Real names are ASCII and case-sensitive, matching
`[A-Za-z][A-Za-z0-9-]*`, with at most **64 characters**. Real `Name` fields
are unique across options and positions because they become dictionary keys.
A short alias is exactly one ASCII letter and is unique among short aliases;
it may match a real name. The long option `help` and short alias `h` are
reserved; positional names `help` and `h` are allowed. Boolean options also
reserve their generated `no-Name` aliases against other long options and
generated negative aliases. A positional name may match a negative alias.
Required positions must precede optional positions.

Explicit defaults must use the matching `Value` variant. Combining
`Required := true` with an explicit default is invalid. Boolean options
without an explicit default yield `Value.Bool(false)` when omitted, unless
required; required Boolean flags must still occur explicitly. Optional values
without defaults are omitted from the result dictionary. Defaults and supplied
values use canonical names as keys; using an option alias does not create a
second key for that option.

## Argument syntax

| Form | Meaning |
|------|---------|
| `--name=value` | supplies a long option's value in the same token |
| `--name value` | supplies its value in the next token |
| `-n value` | uses a short alias and the next token |
| `--flag` / `-f` | sets a Boolean option to true |
| `--flag=false` | explicitly parses a Boolean value |
| `--no-flag` | sets the Boolean option to false |
| `--` | ends option recognition; following tokens are positional values |
| `-` | an ordinary positional value |

There are no short bundles, attached short values, repeated options, or
subcommands. For example, `-ab`, `-n42`, and `-n=42` are not accepted short
forms. Repeating a Boolean flag through its negative alias is also a duplicate.
Negative aliases do not accept `=value`.

A value-taking option consumes the next token even when it starts with `-`,
including `--help`; the exact `--` sentinel is not consumed as a value and
causes `MissingValue`. Inline empty text such as `--name=` is valid for a
Text option. A Boolean flag does not consume a following token as its value;
use the positive long option's inline form for an explicit Boolean value.

Before `--`, a dash-prefixed positional value is recognized as numeric only
when the current positional kind is Int or Float and `-` is followed by an
ASCII digit. Other dash-prefixed tokens are unknown options. Thus `-12` can
be an Int positional, while `-$FF` requires `--` when used positionally.
A named Int option can consume `-$FF` directly as its next-token value.

## Typed values

Text values preserve their argument text. Int, Float, and Bool use
[`Std.Text.Parse.Int`, `Real`, and `Bool`](../text/parse.md), including Pascal
numeric syntax, trimmed parsing, checked bounds, and finite reals. For example,
Int accepts `1_024` and `-$FF`; Float accepts `-2.5` but requires the real
parser's fractional part; Bool accepts `true` and `false` case-insensitively.
This is independent of option-name case sensitivity.

`Outcome.Parsed` contains one dictionary for options and positions. Read a key
with `Values.Get(Name)` and inspect its `Value` variant. Missing optional
values have `None`; provided/default values have `Some(Cli.Value.Kind(...))`.

```pascal id=example-1 run
program CliExample;

uses Std.Cli as Cli, Std.Test as Test;

begin
  const Definition := Cli.Schema(
    ProgramName := 'copy',
    Description := 'Copy one input with a repeat count.',
    Options := [
      Cli.OptionSpec(
        Name := 'count',
        Short := Some('n'),
        Kind := Cli.Kind.Int,
        Default := Some(Cli.Value.Int(1)),
        Help := 'Number of copies'
      ),
      Cli.OptionSpec(Name := 'verbose', Short := Some('v'), Kind := Cli.Kind.Bool)
    ],
    Positionals := [Cli.PositionalSpec(Name := 'input')]
  );
  case Cli.Parse(['-n', '3', '--verbose', 'notes.txt'], Definition) of
    when Ok(Cli.Outcome.Parsed(const Values)):
      Test.AssertTrue(Values.Get('count') is Some(Cli.Value.Int(3)));
      Test.AssertTrue(Values.Get('verbose') is Some(Cli.Value.Bool(true)));
      Test.AssertTrue(Values.Get('input') is Some(Cli.Value.Text('notes.txt')));
    when Ok(Cli.Outcome.Help(_)):
      Test.Fail('Unexpected help request');
    when Error(const Failure):
      Test.Fail(Failure.Message());
  end case;
end.
```

## Help handling

Exact `-h` or `--help` at an option position returns `Ok(Cli.Outcome.Help(Text))`
immediately. Later tokens and missing required arguments are ignored; errors
encountered before help remain errors. Help consumed as an option value or
appearing after `--` is ordinary text. The schema is always validated first.

`Cli.Help(Definition)` generates the same text without parsing arguments.
Both routes return text ending in LF. The caller decides whether to write it
to a console, return it elsewhere, or select an exit status.

## Errors and indexes

All failures are returned as `Error(Cause)`:

| Variant | Fields |
|---------|--------|
| `InvalidSchema` | `Name: string; Reason: string` |
| `UnknownOption` | `Argument: string; Index: integer` |
| `DuplicateOption` | `Name: string; Index: integer` |
| `MissingValue` | `Name: string; Index: integer` |
| `UnexpectedValue` | `Name: string; Index: integer` |
| `InvalidValue` | `Name: string; Text: string; Index: integer; Cause: Parse.ParseError` |
| `MissingRequired` | `Name: string` |
| `UnexpectedArgument` | `Argument: string; Index: integer` |
| `LimitExceeded` | `Resource: string; Limit: integer` |

Indexes are zero-based positions in the supplied argument array. An invalid
value identifies the value token, or the same option token for inline values.
Missing values and duplicates identify the option token. `InvalidValue` retains
the typed `Std.Text.Parse.ParseError` as its cause; it is not replaced by a message
string. Use `.Message()` explicitly when text is needed.

## Limits

| Resource | Limit |
|----------|-------|
| `schema specs` | 256 options and positions combined |
| `schema text` | 1_000_000 Unicode scalars across schema text |
| `arguments` | 4096 argument tokens |
| `argument text` | 1_000_000 Unicode scalars across argument strings |

Text limits count Unicode scalars, not UTF-8 bytes. Exceeding a bound returns
`Cli.CliError.LimitExceeded(Resource, Limit)`.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Public source facade | [`Cli.fpas`](../../../../lib/Std/Cli.fpas) |
| Types and nominal errors | [`Cli/Types.fpas`](../../../../lib/Std/Cli/Types.fpas) |
| Pascal regressions | [`tests/stdlib/cli/`](../../../../tests/stdlib/cli) |

`Std.Cli` is source-defined. Its facade is the editor/API declaration surface;
there is no generated `lib/api/Std/Cli.fpas`.

## See also

- [Process arguments](args.md)
- [Typed parsing](../text/parse.md)
- [Host and I/O index](README.md)
