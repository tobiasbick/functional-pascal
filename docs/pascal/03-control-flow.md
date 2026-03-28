# 3. Control Flow

## If / Then / Else

```pascal
if X > 0 then
  Std.Console.WriteLn('positive')
else if X = 0 then
  Std.Console.WriteLn('zero')
else
  Std.Console.WriteLn('negative');
```

With blocks:

```pascal
if X > 10 then
begin
  Std.Console.WriteLn('large');
  mutable X := X - 10;
end
else
begin
  Std.Console.WriteLn('small');
end;
```

## Case Of

The enhanced `case` statement supports matching on integers, chars, strings, booleans, and enums:

```pascal
case Day of
  'Monday':    Std.Console.WriteLn('Start of week');
  'Friday':    Std.Console.WriteLn('Almost weekend');
  'Saturday',
  'Sunday':    Std.Console.WriteLn('Weekend');
else
  Std.Console.WriteLn('Midweek');
end;
```

With ranges:

```pascal
case Score of
  0..59:    Grade := 'F';
  60..69:   Grade := 'D';
  70..79:   Grade := 'C';
  80..89:   Grade := 'B';
  90..100:  Grade := 'A';
end;
```

## For Loop

### Counting Up

```pascal
for I: integer := 1 to 10 do
begin
  Std.Console.WriteLn(I);
end;
```

### Counting Down

```pascal
for I: integer := 10 downto 1 do
begin
  Std.Console.WriteLn(I);
end;
```

### For-In (Array Iteration)

Iterates over each element of an array. The loop variable is immutable.

```pascal
var
  Names: array of string := ['Alice', 'Bob', 'Charlie'];

for Name: string in Names do
  Std.Console.WriteLn(Name);
```

The element type must match the array's element type:

```pascal
var
  Scores: array of integer := [10, 20, 30];

for S: integer in Scores do
  Std.Console.WriteLn(S);
```

## While Loop

```pascal
mutable var
  Count: integer := 0;

while Count < 10 do
begin
  Std.Console.WriteLn(Count);
  Count := Count + 1;
end;
```

## Repeat-Until Loop

The body executes at least once:

```pascal
mutable var
  Input: string := '';

repeat
  Input := Std.Console.ReadLn();
until Input = 'quit';
```

## Break and Continue

```pascal
for I: integer := 1 to 100 do
begin
  if I mod 2 = 0 then
    continue;

  if I > 50 then
    break;

  Std.Console.WriteLn(I);
end;
```
