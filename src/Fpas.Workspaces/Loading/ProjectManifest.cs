using System.Text.Json.Serialization;

namespace Fpas.Workspaces.Loading;

internal sealed record ManifestReference(string Path, string Configuration, string Platform);
internal sealed record ManifestOutput(string Project, string Path);
internal sealed record ProjectManifest(int Version, string Phase, string Project, string Configuration,
    string Platform, string Framework, string AssemblyName, string Target, string CompilerHash,
    bool ValidateDocumentation, bool WarningsAsErrors, string[] WarningErrors, string[] WarningExceptions, string[] NoWarn,
    string[] Sources, string[] References, ManifestReference[] Projects, ManifestOutput[] Outputs, string BuildCache, string ExpectedBuildInputs, Dictionary<string, string> InputHashes);
internal sealed record BuildProof([property: JsonPropertyName("Inputs")] string Inputs, [property: JsonPropertyName("AssemblyHash")] string AssemblyHash);

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(ProjectManifest))]
[JsonSerializable(typeof(BuildProof))]
internal partial class ProjectManifestJson : JsonSerializerContext;
