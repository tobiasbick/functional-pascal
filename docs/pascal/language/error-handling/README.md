# Errors and resources

Use CLR nullability for absence, application unions for normal domain outcomes,
and CLR exceptions for failures. Resources have explicit lexical cleanup.

| Topic | Description |
| --- | --- |
| [Absence and domain outcomes](absence-and-outcomes.md) | Nullable values, Try APIs, and application unions |
| [Exceptions](exceptions.md) | Raise, ordered catches, filters, rethrow, and finally |
| [Synchronous disposal](resources.md) | Using const, reverse order, scopes, and failed cleanup |
| [Asynchronous disposal](async-disposal.md) | Await using const and completion before scope exit |
| [Contracts](../functions/contracts.md) | Preconditions/postconditions and their exception boundary |

## See also

- [Nullable types](../types/nullable-types.md)
- [Unions](../types/unions.md)
- [Async execution](../concurrency/README.md)
- [Files and streams](../../libraries/files-and-streams.md)
