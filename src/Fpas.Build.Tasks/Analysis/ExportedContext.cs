using System.Text.Json.Serialization;

namespace Fpas.Build.Tasks.Analysis;

internal sealed record ExportedReference(string Path, string Configuration, string Platform);
internal sealed record ExportedOutput(string Project, string Path);
internal sealed record ExportedContext(int Version, string Phase, string Project, string Configuration,
    string Platform, string Framework, string AssemblyName, string Target, string CompilerHash,
    bool ValidateDocumentation, bool WarningsAsErrors, string[] WarningErrors, string[] WarningExceptions, string[] NoWarn,
    string[] Sources, string[] References, ExportedReference[] Projects, ExportedOutput[] Outputs, string BuildCache, string ExpectedBuildInputs, Dictionary<string, string> InputHashes);

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(ExportedContext))]
internal partial class ExportedContextJson : JsonSerializerContext;
