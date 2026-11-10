# CLR collection events

This program subscribes a capturing FPAS delegate to the actual .NET
BindingList<int>.ListChanged event. It removes the same delegate before the second
addition, so only the first change invokes the handler.

After the [local SDK bootstrap](../../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet restore examples/interop/events/Events.fpasproj
dotnet build examples/interop/events/Events.fpasproj --no-restore
dotnet examples/interop/events/bin/Debug/net10.0/Events.dll
```

Expected output:

```text
ItemAdded
1
```
