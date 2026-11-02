using System.Reflection;

namespace Fpas.Tests.Support;

internal static class TestEnvironment
{
    internal static string Metadata(string key) => typeof(TestEnvironment).Assembly
        .GetCustomAttributes<AssemblyMetadataAttribute>().Single(attribute => attribute.Key == key).Value
        ?? throw new InvalidOperationException($"Missing test metadata: {key}.");

    internal static string ReferenceDirectory => Metadata("FpasReferenceDirectory");
    internal static string[] References => Directory.GetFiles(ReferenceDirectory, "*.dll");
    internal static string DotnetHost => Environment.GetEnvironmentVariable("DOTNET_HOST_PATH") ?? "dotnet";
}
