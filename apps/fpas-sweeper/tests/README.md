# fpas-sweeper tests

Run the FPAS xUnit tests from the application directory after setting up the
[local FPAS SDK](../../../docs/build/development.md#build-an-fpas-application).

```sh
dotnet test tests/FpasSweeper.Tests/FpasSweeper.Tests.fpasproj --filter 'Category!=Gui'
```

The [test project](FpasSweeper.Tests/FpasSweeper.Tests.fpasproj) checks the
[game model](../doc/README.md#game-model) and its CUI desktop UI.
[BoardTests.fpas](FpasSweeper.Tests/BoardTests.fpas) covers input bounds, flags,
first-reveal safety, flood reveal, rectangular geometry and round outcomes.
[InputTests.fpas](FpasSweeper.Tests/InputTests.fpas) checks the reveal and flag
messages and the keyboard shortcuts.

Run the native tests on a desktop display or under Xvfb.

```sh
dotnet test tests/FpasSweeper.Tests/FpasSweeper.Tests.fpasproj --filter 'Category=Gui'
```

[NativeBoardTests.fpas](FpasSweeper.Tests/NativeBoardTests.fpas) drives both
mouse buttons through CUI after a window resize. It checks cell gaps, flags,
and which tiles the mouse opens.
