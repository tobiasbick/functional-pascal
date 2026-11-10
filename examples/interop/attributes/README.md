# CLR custom attributes

The program annotates a record field with the actual .NET JsonPropertyNameAttribute.
The compiler places it on the emitted property; System.Text.Json uses that name
when serializing the record. Attribute constructors do not run during compilation.

After the [local SDK bootstrap](../../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet restore examples/interop/attributes/Attributes.fpasproj
dotnet build examples/interop/attributes/Attributes.fpasproj --no-restore
dotnet examples/interop/attributes/bin/Debug/net10.0/Attributes.dll
```

Expected output:

```json
{"sample_name":"Alpha","Value":42}
```
