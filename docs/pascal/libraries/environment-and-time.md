# Environment settings, instants, and durations

Use actual System.Environment, DateTimeOffset, TimeSpan, and Stopwatch APIs.
Configuration parsing and persisted timestamps should state their culture and
format explicitly so they behave consistently on Windows and Linux.

## Read and validate an environment setting

[GetEnvironmentVariable(String)](https://learn.microsoft.com/en-us/dotnet/api/system.environment.getenvironmentvariable?view=net-10.0)
returns a nullable string from the current process. This example uses three
attempts when FPAS_RECIPE_ATTEMPTS is absent, accepts integers from one through
ten, and reports invalid input without printing the supplied value.
NumberStyles.Integer permits a sign and surrounding whitespace; the range
check is a separate application rule.

```pascal
program EnvironmentConfiguration;

uses System as Sys, System.Console as Console, System.Globalization as Global;

function ReadAttempts(Text: string?): int;
begin
  if Text = null then
    return 3;
  end if;
  var Value: int := 0;
  if not int.TryParse(
    Text, Global.NumberStyles.Integer,
    Global.CultureInfo.InvariantCulture, out Value
  ) or Value < 1 or Value > 10
  then
    raise new Sys.FormatException('Invalid FPAS_RECIPE_ATTEMPTS');
  end if;

  return Value;
end function;

begin
  try
    const Attempts: int := ReadAttempts(
      Sys.Environment.GetEnvironmentVariable('FPAS_RECIPE_ATTEMPTS')
    );
    Console.WriteLine(Attempts);
  catch Failure: Sys.FormatException do
    Console.Error.WriteLine(Failure.Message);
    Sys.Environment.ExitCode := 2;
  end try;
end;

end program;
```

Set the variable in the launching shell, or pass it through ProcessStartInfo's
Environment dictionary as shown in the [process recipe](processes.md).
Changing that dictionary affects only the child. Environment.SetEnvironmentVariable
changes process-wide state when using its process overload; it is unsuitable
for concurrent request-local configuration. Environment variable names are
case-sensitive on Unix-like systems; use the same spelling on both platforms.

The Markdown test supplies valid values, removes the variable for the default
path, and checks invalid text, bounds, and Int32 overflow in separate child
processes. No user/machine environment setting is changed.

## Store an instant and calculate a deadline

DateTimeOffset records an instant with a UTC offset. An offset is not a time-zone
identity or a daylight-saving rule. Use UTC for a shared time basis, retain an
offset when it belongs to the data, and use TimeSpan for a duration. The example
adds ninety minutes of elapsed time without consulting the machine's local zone.

```pascal
program TimeAndDuration;

uses
  System as Sys, System.Console as Console, System.Diagnostics as Diagnostics,
  System.Globalization as Global;

begin
  const Culture: Global.CultureInfo := Global.CultureInfo.InvariantCulture;
  const Started: long := Diagnostics.Stopwatch.GetTimestamp();
  const Stamp: Sys.DateTimeOffset := Sys.DateTimeOffset.ParseExact(
    '2030-04-05T12:30:00.0000000+02:00', 'O', Culture, Global.DateTimeStyles.None
  );
  const Wire: string := Stamp.ToString('O', Culture);
  const Restored: Sys.DateTimeOffset := Sys.DateTimeOffset.ParseExact(
    Wire, 'O', Culture, Global.DateTimeStyles.None
  );
  const Utc: Sys.DateTimeOffset := Stamp.ToUniversalTime();
  const Window: Sys.TimeSpan := Sys.TimeSpan.FromMinutes(90.0);
  const Deadline: Sys.DateTimeOffset := Stamp + Window;
  const Duration: Sys.TimeSpan := Deadline - Stamp;
  const Elapsed: Sys.TimeSpan := Diagnostics.Stopwatch.GetElapsedTime(Started);
  const Now: Sys.DateTimeOffset := Sys.DateTimeOffset.UtcNow;
  if Restored <> Stamp or Restored.Offset <> Stamp.Offset
    or Utc.Hour <> 10 or Utc.Offset <> Sys.TimeSpan.Zero
    or Duration <> Window or Deadline - Window <> Stamp or Elapsed < Sys.TimeSpan.Zero
    or Now.Offset <> Sys.TimeSpan.Zero
  then
    raise new Sys.InvalidOperationException('Incorrect instant or duration');
  end if;

  Console.WriteLine(Utc.ToString('O', Culture));
  Console.WriteLine(Duration.ToString('c', Culture));
end;

end program;
```

The program prints `2030-04-05T10:30:00.0000000+00:00` and `01:30:00`.
The string-based
[ParseExact](https://learn.microsoft.com/en-us/dotnet/api/system.datetimeoffset.parseexact?view=net-10.0)
overload takes a format, provider, and DateTimeStyles; malformed input throws
FormatException. The round-trip `O` format preserves fractional seconds and the
offset. TimeSpan's `c` format represents a duration without calendar/time-zone rules.
Subtracting two DateTimeOffset values yields a TimeSpan. Adding or subtracting
a TimeSpan yields a DateTimeOffset.

Use UtcNow for a current wall-clock instant and
[Stopwatch.GetElapsedTime(Int64)](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.stopwatch.getelapsedtime?view=net-10.0)
for elapsed work measured from GetTimestamp. The example checks no exact current
time or elapsed duration and performs no sleeps. It demonstrates the APIs rather
than a benchmark; see [profiling](../tools/profiling.md) for measurement boundaries.

## See also

- [Library guides](README.md)
- [Numbers and random values](numbers-and-random.md)
- [Nullability and outcomes](../language/error-handling/absence-and-outcomes.md)
- [DateTimeOffset reference](https://learn.microsoft.com/en-us/dotnet/api/system.datetimeoffset?view=net-10.0)
- [TimeSpan reference](https://learn.microsoft.com/en-us/dotnet/api/system.timespan?view=net-10.0)
