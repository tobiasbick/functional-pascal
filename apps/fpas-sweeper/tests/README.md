# fpas-sweeper tests

Run the FPAS xUnit tests from the application directory after setting up the
[local FPAS SDK](../../../docs/build/development.md#build-an-fpas-application).

```sh
dotnet test tests/FpasSweeper.Tests/FpasSweeper.Tests.fpasproj
```

The [test project](FpasSweeper.Tests/FpasSweeper.Tests.fpasproj) checks the
[game model](../doc/README.md#game-model) without opening a desktop window.
[BoardTests.fpas](FpasSweeper.Tests/BoardTests.fpas) covers input bounds, flags,
first-reveal safety, flood reveal, rectangular geometry and round outcomes.
