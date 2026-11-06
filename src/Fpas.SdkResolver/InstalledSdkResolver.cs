using Microsoft.Build.Framework;
using System.Text.Json;

namespace Fpas.SdkResolver;

/// <summary>MSBuild-hosted package SDK resolution that never contacts a feed.</summary>
public sealed class InstalledSdkResolver : Microsoft.Build.Framework.SdkResolver
{
    public override string Name => "FPAS installed SDKs";
    public override int Priority => 10001;

    public override SdkResult Resolve(SdkReference sdk, SdkResolverContext context, SdkResultFactory factory)
    {
        try
        {
            var version = sdk.Version;
            if (string.IsNullOrEmpty(version))
                for (var directory = new DirectoryInfo(Path.GetDirectoryName(context.ProjectFilePath)!); directory is not null; directory = directory.Parent)
                {
                    var global = Path.Combine(directory.FullName, "global.json");
                    if (!File.Exists(global)) continue;
                    using var json = JsonDocument.Parse(File.ReadAllBytes(global));
                    if (json.RootElement.TryGetProperty("msbuild-sdks", out var sdks) && sdks.TryGetProperty(sdk.Name, out var value)) version = value.GetString();
                    break;
                }
            var cache = Environment.GetEnvironmentVariable("FPAS_INSTALLED_SDK_CACHE");
            if (!string.IsNullOrEmpty(cache) && !string.IsNullOrEmpty(version) && Safe(sdk.Name) && Safe(version))
            {
                var normalized = version.Split('+')[0];
                var parts = normalized.Split('-', 2);
                if (Version.TryParse(parts[0], out var number))
                    normalized = number.Major + "." + number.Minor + "." + Math.Max(0, number.Build) +
                        (number.Revision > 0 ? "." + number.Revision : "") + (parts.Length == 2 ? "-" + parts[1] : "");
                var folder = Path.Combine(cache, sdk.Name.ToLowerInvariant(), normalized.ToLowerInvariant());
                if (File.Exists(Path.Combine(folder, ".nupkg.metadata")) && Directory.Exists(Path.Combine(folder, "Sdk")))
                    return factory.IndicateSuccess(Path.Combine(folder, "Sdk"), version);
            }
            return factory.IndicateFailure(["Package SDK is not installed. Restore/build the project explicitly before analysis: " + sdk.Name]);
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or JsonException or ArgumentException or InvalidOperationException)
        { return factory.IndicateFailure(["Cannot resolve an installed SDK: " + error.Message]); }
    }

    private static bool Safe(string value) => value is not ("." or "..") && value.IndexOfAny(['/', '\\', ':', '$']) < 0;
}
