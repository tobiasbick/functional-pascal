# `Std.Calendar`

Pure Gregorian dates, UTC date-times, strict text conversion and checked arithmetic.
Import with `uses Std.Calendar as Calendar;`. The unit reads no clock, locale,
environment variable or time-zone database.

```pascal id=example-calendar run
program CalendarExample;

uses Std.Calendar as Calendar, Std.Test as Test;

begin
  const Leap := Calendar.CreateDate(2000, 2, 29).Unwrap();
  const Next := Calendar.AddDays(Leap, 1).Unwrap();
  Test.AssertEquals('2000-03-01', Calendar.FormatDate(Next));
  Test.AssertEquals(1, Calendar.DaysBetween(Leap, Next));
  const Moment := Calendar.ParseDateTime('1970-01-01T00:30:00.123+01:00').Unwrap();
  Test.AssertEquals('1969-12-31T23:30:00.123Z', Calendar.FormatDateTime(Moment));
  Test.AssertEquals(-1_799_877, Calendar.ToTimestampMillis(Moment));
end.
```

## Values and range

`Calendar.Date` is a Gregorian calendar date in years `1..9999`.
The Gregorian leap-year rule applies to the entire range, including years before
1582: a year is a leap year when divisible by four, except centuries not divisible
by 400. Dates do not skip days at historical calendar reforms. Year zero is invalid.

`Calendar.DateTime` is a UTC date and time with millisecond precision. Hours are
`0..23`, minutes and seconds `0..59`, and milliseconds `0..999`. Leap seconds and
`24:00:00` are invalid. The supported interval is
`0001-01-01T00:00:00.000Z` through `9999-12-31T23:59:59.999Z`, inclusive.

Both types have private fields. Applications obtain values through the checked
constructors, parsers, arithmetic or timestamp conversion; they cannot construct
or update the fields directly. Copying and whole-value equality are supported.

| Value | Read-only methods |
|-------|-------------------|
| `Date` | `Year(): integer`, `Month(): integer`, `Day(): integer` |
| `DateTime` | `ToDate(): Date`, `Hour(): integer`, `Minute(): integer`, `Second(): integer`, `Millisecond(): integer` |

For example, `Moment.ToDate().Year()` obtains the UTC year. To change a component,
call a constructor with the desired components; there are no mutating setters.

## Quick reference

The signatures below use the imported `Calendar.Date`, `Calendar.DateTime` and
`Calendar.CalendarError` types. `Result` failures are ordinary typed values.

| Function | Result |
|----------|--------|
| `CreateDate(Year: integer; Month: integer; Day: integer)` | `Result of (Date, CalendarError)` |
| `CreateDateTime(Year: integer; Month: integer; Day: integer; Hour: integer; Minute: integer; Second: integer; Millisecond: integer)` | `Result of (DateTime, CalendarError)` |
| `ParseDate(Text: string)` | `Result of (Date, CalendarError)` |
| `ParseDateTime(Text: string)` | `Result of (DateTime, CalendarError)` |
| `FormatDate(Value: Date)` | `string` |
| `FormatDateTime(Value: DateTime)` | `string` |
| `FromTimestampMillis(Value: integer)` | `Result of (DateTime, CalendarError)` |
| `ToTimestampMillis(Value: DateTime)` | `integer` |
| `AddDays(Value: Date; Days: integer)` | `Result of (Date, CalendarError)` |
| `DaysBetween(Start: Date; Finish: Date)` | `integer` |
| `AddMillis(Value: DateTime; Milliseconds: integer)` | `Result of (DateTime, CalendarError)` |
| `MillisBetween(Start: DateTime; Finish: DateTime)` | `integer` |

Constructors validate every component. `CreateDateTime` checks the date before
the time. Formatters and getters need no Result because their inputs are already
valid values.

## Text grammar

`ParseDate` accepts exactly `YYYY-MM-DD`, with four ASCII year digits and two
ASCII digits each for month and day. It rejects whitespace, signed years,
week dates, ordinal dates and trailing text. Its maximum input length is ten
Unicode scalars.

`ParseDateTime` accepts the following RFC-3339 timestamp forms:

```text
YYYY-MM-DDTHH:mm:ssZ
YYYY-MM-DDTHH:mm:ss.fZ
YYYY-MM-DDTHH:mm:ss.ffZ
YYYY-MM-DDTHH:mm:ss.fffZ
```

`T` and `Z` may be lowercase on input. A known numeric UTC offset `+HH:MM` or
`-HH:MM` may replace `Z`; its hour is `00..23` and minute `00..59`. The parser
subtracts the signed offset from the supplied local fields and stores UTC.
Both the supplied date and normalized UTC result must fit the supported range.
`+00:00` is valid. RFC-3339 `-00:00` means an unknown local offset and is rejected.

Fractions contain one to three ASCII digits: `.1` is 100 milliseconds and `.12`
is 120 milliseconds. A bare decimal point or more than three fractional digits
is invalid, even when the extra digits are zero. The parser never rounds or
truncates fractional seconds. A zone designator is required; spaces, a decimal
comma and offset seconds are not accepted. The maximum input length is 29
Unicode scalars. Length checks precede component parsing, and errors do not
retain an unbounded copy of the input.

`FormatDate` always produces `YYYY-MM-DD`.
`FormatDateTime` always produces `YYYY-MM-DDTHH:mm:ss.SSSZ`, including `.000`
for a whole second. Output uses ASCII digits, uppercase `T`/`Z` and no locale
settings. The original input offset is not retained.

```pascal id=example-negative-epoch run
program NegativeEpoch;

uses Std.Calendar as Calendar, Std.Test as Test;

begin
  const Before := Calendar.FromTimestampMillis(-1).Unwrap();
  Test.AssertEquals('1969-12-31T23:59:59.999Z', Calendar.FormatDateTime(Before));
  Test.AssertEquals('1970-01-01T00:00:00.000Z',
    Calendar.FormatDateTime(Calendar.AddMillis(Before, 1).Unwrap()));
  Test.AssertTrue(
    Calendar.ParseDateTime('2000-01-01T00:00:00-00:00')
      is Error(Calendar.CalendarError.InvalidOffset('-00:00'))
  );
end.
```

## Timestamp conversion and arithmetic

Unix milliseconds are measured from `1970-01-01T00:00:00Z`. Negative values
represent earlier instants; negative division is handled as a floor day and a
nonnegative time of day. The inclusive timestamp bounds are
`-62_135_596_800_000` and `253_402_300_799_999`.

`DaysBetween(Start, Finish)` returns Finish minus Start in whole calendar days.
`MillisBetween(Start, Finish)` returns Finish minus Start in milliseconds.
Both may be negative, and every possible difference fits an FPAS integer.
`AddDays` and `AddMillis` accept positive, zero or negative deltas. They check
the result range before addition, including for minimum and maximum integer
arguments. A range failure returns `OutOfRange`, without wrapping, clamping or
a runtime arithmetic failure.

Month/year addition, named time zones, daylight-saving rules and localized
presentation are separate extensions. UTC days here always contain 86,400
seconds. Calendar values do not support ordering operators; compare timestamps
or signed differences, and use `SortBy` with a timestamp key for date-times.

Use the existing wall clock explicitly when the current time is needed:

```pascal id=example-wall-clock
program CurrentUtc;

uses Std.Calendar as Calendar, Std.Console as Console, Std.Time as Time;

begin
  const Now := Calendar.FromTimestampMillis(Time.TimestampMillis()).Unwrap();
  Console.WriteLn(Calendar.FormatDateTime(Now));
end.
```

Monotonic timer readings are not Unix timestamps and cannot be used for this
conversion. The clock and sleep behavior of [`Std.Time`](time.md) is unchanged.
Pure calendar calls are permitted in seeded debugging and recording capture.

## Typed failures

| Variant | Meaning |
|---------|---------|
| `InvalidDate(Year: integer; Month: integer; Day: integer)` | A date component or Gregorian day is invalid. |
| `InvalidTime(Hour: integer; Minute: integer; Second: integer; Millisecond: integer)` | A time component is outside its supported range. |
| `InvalidSyntax(Offset: integer; Reason: string)` | Malformed text; Offset is a zero-based Unicode scalar index or the position where input ended. |
| `InvalidOffset(Text: string)` | A six-character numeric offset is out of range or is the unknown `-00:00` form. |
| `OutOfRange(Operation: string)` | Timestamp conversion, arithmetic or UTC normalization leaves years 1..9999. |

Operation names are `Std.Calendar.FromTimestampMillis`, `Std.Calendar.AddDays`,
`Std.Calendar.AddMillis` or `Std.Calendar.ParseDateTime`. Match the variants or
use `Message()` at a text boundary. No calendar failure fabricates a host cause.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Public source API | [`Calendar.fpas`](../../../../lib/Std/Calendar.fpas) |
| Checked values and arithmetic | [`Values.fpas`](../../../../lib/Std/Calendar/Values.fpas) |
| Parsing and formatting | [`Text.fpas`](../../../../lib/Std/Calendar/Text.fpas) |
| Public source exports | [`stdlib.fpasprj`](../../../../lib/stdlib.fpasprj) |

## See also

- [Host and I/O index](README.md)
- [Clock and sleep helpers](time.md)
- [Native error families](../../language/types/errors.md)
