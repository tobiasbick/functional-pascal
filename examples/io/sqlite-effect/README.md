# Query SQLite with an effect

`FindTitle` describes a parameterized query for a book ID. `Run` supplies the
connection string and cancellation token. Each effect run owns and disposes
its connection and command. Its result distinguishes a title, a missing row,
and a SQLite provider failure.

The program keeps one connection open while it seeds and queries a shared
in-memory database. Microsoft.Data.Sqlite executes its asynchronous ADO.NET
methods synchronously. With this provider, forward and check cancellation
between calls. An in-flight query can hold its thread until it returns.

Run the example after the [local SDK bootstrap](../../../docs/build/development.md#build-an-fpas-application).

```sh
fpas run examples/io/sqlite-effect/SqliteEffect.fpasproj
```

```text
Functional Pascal
Missing book 2
```

## Source files

- [SqliteEffect.fpasproj](SqliteEffect.fpasproj)
- [main.fpas](main.fpas)
