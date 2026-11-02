# Game actions

This executable data-model example combines immutable records and unions. A move
produces Moved(Position), Blocked(Reason), or ItemFound(Name, At). Stayed is a
fieldless case. Describe uses an exhaustive case expression with explicitly
typed payload bindings, including named payload patterns.

These are ordinary outcomes. Blocked does not throw an exception or cause an
implicit return in the caller; the caller inspects the returned data explicitly.

After building and locally packaging the compiler as described in
[MSBuild integration](../../docs/implementation/msbuild.md#local-sdk-bootstrap):

```sh
dotnet restore examples/game-actions/GameActions.fpasproj
dotnet build examples/game-actions/GameActions.fpasproj --no-restore
dotnet examples/game-actions/bin/Debug/net10.0/GameActions.dll
```

Expected output:

```text
Moved to 1,0
Blocked: wall
Found key at 2,0
Stayed
```

See [unions and matching](../../docs/implementation/unions-and-matching.md) for
supported patterns, CLR representation, and remaining limits. This example is a
small simulation and does not build the separate dungeon game repository.
