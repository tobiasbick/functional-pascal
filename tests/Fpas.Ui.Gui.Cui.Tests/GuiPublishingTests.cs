using System.Security.Cryptography;
using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Runs the public FPAS gallery through a package-only consumer in every delivery mode.</summary>
public sealed partial class GuiPublishingTests
{
    /// <summary>Checks the complete host, transitive native assets, model state and isolated executable delivery.</summary>
    [GuiPublishingFact]
    public async Task PublicGalleryRunsInAllFivePublishModes()
    {
        using var fixture = new GuiPackageFixture();
        await fixture.Prepare();
        fixture.Environment["FpasCuiCMakeCommand"] = "unavailable-native-gui-build-tool";
        string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        string output = fixture.File("published");
        string shared = OperatingSystem.IsWindows() ? "fpas_cui.dll" : "libfpas_cui.so";
        string executable = OperatingSystem.IsWindows() ? "GalleryProbe.exe" : "GalleryProbe";
        string nativeHash = Hash(Path.Combine(fixture.Binding, "native", fixture.Rid, "shared", shared));
        var report = new List<object>();
        foreach (string mode in new[] { "framework-dependent", "apphost", "self-contained", "single-file", "aot" })
        {
            await fixture.Check([fixture.Compiler, "publish", fixture.Project, "--mode", mode, "--runtime", fixture.Rid,
                "--configuration", configuration, "--output", output], "publish-" + mode, TimeSpan.FromMinutes(8));
            bool bundled = mode is "single-file" or "aot";
            Assert.Equal(!bundled, File.Exists(Path.Combine(output, "GalleryProbe.dll")));
            Assert.Equal(!bundled, File.Exists(Path.Combine(output, shared)));
            Assert.Equal(6, Directory.GetFiles(Path.Combine(output, "licenses", "cui"), "LICENSE-*.txt").Length);
            if (!bundled) Assert.Equal(nativeHash, Hash(Path.Combine(output, shared)));
            string entry = mode == "framework-dependent" ? "GalleryProbe.dll" : executable;
            string program = Path.Combine(output, entry);
            if (bundled)
            {
                string isolated = Directory.CreateDirectory(fixture.File(mode + "-executable-only")).FullName;
                string copy = Path.Combine(isolated, entry);
                File.Copy(program, copy);
                if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(copy, File.GetUnixFileMode(program));
                Assert.Single(Directory.GetFiles(isolated));
                program = copy;
            }
            string working = Directory.CreateDirectory(fixture.File(mode + "-run-elsewhere")).FullName;
            string extraction = Directory.CreateDirectory(fixture.File(mode + "-extraction")).FullName;
            var runEnvironment = new Dictionary<string, string?>(fixture.Environment)
            {
                ["DOTNET_BUNDLE_EXTRACT_BASE_DIR"] = extraction,
                ["FpasCuiCMakeCommand"] = null
            };
            if (mode is "self-contained" or "single-file" or "aot")
            {
                runEnvironment["PATH"] = "";
                foreach (string key in new[] { "DOTNET_ROOT", "DOTNET_ROOT_X64", "DOTNET_HOST_PATH" })
                    runEnvironment[key] = fixture.File("absent-dotnet");
            }
            string imports = "";
            if (mode == "aot")
            {
                imports = await NativeDependencies.Read(program);
                File.WriteAllText(Path.Combine(fixture.Artifacts, "aot-imports.txt"), imports);
                Assert.DoesNotContain("fpas_cui", imports, StringComparison.OrdinalIgnoreCase);
                Assert.DoesNotContain("coreclr", imports, StringComparison.OrdinalIgnoreCase);
                Assert.Contains(OperatingSystem.IsWindows() ? "opengl32.dll" : "libX11.so.6", imports, StringComparison.OrdinalIgnoreCase);
            }
            foreach (string renderer in new[] { "software", "opengl" })
            {
                string png = Path.Combine(fixture.Artifacts, mode + "-" + renderer + ".png");
                var run = mode == "framework-dependent"
                    ? await ProcessRunner.RunAsync([program, renderer, png], workingDirectory: working,
                        environment: runEnvironment, timeLimit: TimeSpan.FromSeconds(30))
                    : await ProcessRunner.RunExecutableAsync(program, [renderer, png], workingDirectory: working,
                        environment: runEnvironment, timeLimit: TimeSpan.FromSeconds(30));
                File.WriteAllText(Path.Combine(fixture.Artifacts, mode + "-" + renderer + ".log"), run.Output + run.Error);
                Assert.True(run.ExitCode == 0, mode + "/" + renderer + ": " + run.Output + run.Error);
                Assert.Equal("packaged gallery verified: " + renderer + Environment.NewLine, run.Output);
                Assert.Contains(0xff0000ffu, PngAssertions.ReadPixels(png, 1000, 760));
                report.Add(new { mode, renderer, native = bundled ? mode == "aot" ? "static" : "extracted" : "shared",
                    screenshot = Path.GetFileName(png), executableOnly = bundled });
            }
            if (mode == "single-file")
                Assert.Equal(nativeHash, Hash(Assert.Single(Directory.GetFiles(extraction, shared, SearchOption.AllDirectories))));
            else Assert.Empty(Directory.EnumerateFileSystemEntries(extraction));
            File.WriteAllText(Path.Combine(fixture.Artifacts, "delivery-matrix.json"),
                JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }));
        }
    }

    private static string Hash(string file)
    {
        using var stream = File.OpenRead(file);
        return Convert.ToHexString(SHA256.HashData(stream));
    }
}
