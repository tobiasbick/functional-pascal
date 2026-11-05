using Fpas.Compiler;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Diagnostics;

internal sealed class SeverityConverter() : JsonStringEnumConverter<DiagnosticSeverity>(JsonNamingPolicy.CamelCase);
internal sealed class ApplicabilityConverter() : JsonStringEnumConverter<DiagnosticApplicability>(JsonNamingPolicy.CamelCase);

internal sealed record CompileReport(bool Success, string? Output, IReadOnlyList<CompilerDiagnostic> Diagnostics);

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(CompileReport))]
internal partial class CompileJson : JsonSerializerContext;
