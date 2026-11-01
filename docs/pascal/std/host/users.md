# Std.Os.Users

`Std.Os.Users` reads user and group inventories through the bundled sysinfo library.

Import with `uses Std.Os.Users as Users;`.

## Quick reference

| Kind | Signature | Description |
| --- | --- | --- |
| function | `ListUsers(): result of (array of UserInfo, Std.Os.System.SystemError)` | Refresh users and their group memberships |
| function | `ListGroups(): result of (array of GroupInfo, Std.Os.System.SystemError)` | Refresh groups |

`GroupInfo` has strings `Id` and `Name`. `UserInfo` has strings `Id`, `GroupId`
and `Name`, and `Groups: array of GroupInfo`.

IDs are text because OS account identities differ. A Windows user ID can be a
SID; Unix IDs are decimal text. Values are identifiers, not portable account
names or permission checks. Compare IDs on the same host and platform only.

Each query refreshes the inventory. Results are sorted by textual ID. Group
membership order follows the backend. Permissions and OS limitations can
produce incomplete or empty successful inventories. Errors returned by the
backend are preserved; the wrapper cannot identify failures the backend hides.
The API changes no account, membership or permission.

## Example

```pascal id=count-users
program CountUsers;
uses Std.Os.Users as Users, Std.Console as Console;
begin
  const Accounts := Users.ListUsers().Unwrap();
  Console.WriteLn(Accounts.Length());
end.
```

## Implementation (contributors)

Declarations are shared in `crates/fpas-builtins/src/host_info.rs`; collection
is in `crates/fpas-runtime/src/execution/vm/hosted/host_info/`.
Complete editor declarations are in `lib/api/Std/Os.Users.fpas`.

## See also

- [Collection lifetime, errors and platform coverage](system.md)
- [Process user and group IDs](proc.md#read-only-process-inspection)
