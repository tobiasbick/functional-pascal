# CLR custom attributes

The program annotates a record field with the actual .NET JsonPropertyNameAttribute.
The compiler places it on the emitted property; System.Text.Json uses that name
when serializing the record. Attribute constructors do not run during compilation.

After the [local SDK bootstrap](../../docs/implementation/msbuild.md#local-sdk-bootstrap):

```sh
dotnet restore examples/attributes/Attributes.fpasproj
dotnet build examples/attributes/Attributes.fpasproj --no-restore
dotnet examples/attributes/bin/Debug/net10.0/Attributes.dll
```

Expected output:

```json
{"player_name":"Mara","Score":42}
```
