using System.IO.Compression;
using System.Runtime.InteropServices;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class LocalDeploymentTests
{
    private static void CheckCuiAssets(string package, string checkout)
    {
        bool windows = OperatingSystem.IsWindows();
        string rid = windows ? "win-x64" : "linux-x64";
        string shared = windows ? "fpas_cui.dll" : "libfpas_cui.so";
        string archive = windows ? "fpas_cui.lib" : "libfpas_cui.a";
        using var contents = ZipFile.OpenRead(package);
        var unpacked = Path.GetDirectoryName(package)!;
        var library = Path.Combine(unpacked, shared);
        var sharedEntry = contents.GetEntry($"tools/gui/runtimes/{rid}/native/{shared}");
        Assert.NotNull(sharedEntry);
        sharedEntry.ExtractToFile(library);
        var handle = NativeLibrary.Load(library);
        try { Assert.NotEqual(IntPtr.Zero, NativeLibrary.GetExport(handle, "fpas_cui_wake_post")); }
        finally { NativeLibrary.Free(handle); }
        var archiveEntry = contents.GetEntry($"tools/gui/runtimes/{rid}/static/{archive}");
        Assert.NotNull(archiveEntry);
        using (var stream = archiveEntry.Open())
        {
            Span<byte> magic = stackalloc byte[8];
            stream.ReadExactly(magic);
            Assert.True(magic.SequenceEqual("!<arch>\n"u8));
        }
        foreach (string name in new[] { "CUI", "Geist", "Inter", "Lucide", "stb", "Unicode" })
        {
            string filename = $"LICENSE-{name}.txt";
            var entry = contents.GetEntry("tools/gui/licenses/" + filename);
            Assert.NotNull(entry);
            using var notice = new StreamReader(entry.Open());
            Assert.Equal(File.ReadAllText(Path.Combine(checkout, "third-party", "cui", filename)), notice.ReadToEnd());
        }
    }

    private static async Task CheckMissingCuiAssets(string sdkProject, string checkout, string output,
        IReadOnlyDictionary<string, string?> environment)
    {
        bool windows = OperatingSystem.IsWindows();
        string otherRid = windows ? "linux-x64" : "win-x64";
        string artifactDirectory = Path.Combine(checkout, ".temp-data", "dist", windows ? "windows" : "unix");
        var pack = await ProcessRunner.RunAsync(["pack", sdkProject, "--no-build", "--no-restore", "-c", "Release",
            "--artifacts-path", artifactDirectory, "--output", output, "-p:FpasCuiDistributionRid=" + otherRid],
            workingDirectory: checkout, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.NotEqual(0, pack.ExitCode);
        Assert.Contains($"Missing CUI asset for RID '{otherRid}'", pack.Output + pack.Error);
        Assert.False(Directory.Exists(output) && Directory.EnumerateFiles(output, "*.nupkg").Any());
    }
}
