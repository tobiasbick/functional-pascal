# Inventory model

This executable example uses nominal immutable records for items and inventory
positions. A position has a default quantity and an instance method. Drinking a
potion creates a new position with `with`; the original position and its item
remain available. The unit function sums positions through the real .NET
IEnumerable interface.

After building and locally packaging the compiler as described in
[MSBuild integration](../../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet restore examples/types/inventory-model/InventoryModel.fpasproj
dotnet build examples/types/inventory-model/InventoryModel.fpasproj --no-restore
dotnet examples/types/inventory-model/bin/Debug/net10.0/InventoryModel.dll
```

With an invariant or English numeric culture, output is:

```text
Before: 3
After: 2
Weight: 2
Original unchanged: True
Item retained: True
```

See [records](../../../docs/pascal/language/types/records.md) for supported
construction, equality, private state, and cross-assembly behavior. This is a
small data model; the dungeon game is not built by this example.
