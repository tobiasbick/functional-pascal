using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;
using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.Gui.Cui.Tests;

public sealed partial class FpasProjectTests
{
    /// <summary>Verifies signed CUI delivery and runs both renderers in every FPAS publish mode.</summary>
    [GuiPublishingFact]
    [Trait("Category", "NativeGui")]
    public async Task PackagedGuiRunsInEveryPublishMode()
    {
        // The Linux Native AOT linker targets pass NuGet archive paths without quoting them.
        using var directory = new TemporaryDirectory(prefix: "fpas-gui-aot-");
        string rid = OperatingSystem.IsWindows() ? "win-x64" : "linux-x64";
        var (project, sdk, environment) = await PrepareGuiProject(directory, rid);
        string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        await CheckPackagedBuild(project, configuration, rid, environment, directory);

        using var key = RSA.Create(2048);
        var request = new CertificateRequest("CN=FPAS GUI publishing test", key, HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1);
        using var certificate = request.CreateSelfSigned(DateTimeOffset.UtcNow.AddMinutes(-1), DateTimeOffset.UtcNow.AddDays(1));
        string pfx = directory.File("publisher.pfx");
        string publicKey = directory.File("publisher.cer");
        string password = Guid.NewGuid().ToString("N");
        const string passwordVariable = "FPAS_TEST_GUI_PFX_PASSWORD";
        File.WriteAllBytes(pfx, certificate.Export(X509ContentType.Pfx, password));
        File.WriteAllBytes(publicKey, certificate.Export(X509ContentType.Cert));
        var publishEnvironment = new Dictionary<string, string?>(environment) { [passwordVariable] = password };

        string compiler = Path.Combine(sdk, "tools", "compiler", "Fpas.Cli.dll");
        string output = directory.File("published");
        string package = Path.Combine(environment["NUGET_PACKAGES"]!, "fpas.gui.cui", Path.GetFileName(sdk));
        string sharedName = OperatingSystem.IsWindows() ? "fpas_cui.dll" : "libfpas_cui.so";
        string sharedHash = FileHash(Path.Combine(package, "native", rid, "shared", sharedName));
        string[] notices = Directory.GetFiles(Path.Combine(package, "licenses", "cui"), "LICENSE-*.txt");
        Assert.Equal(6, notices.Length);

        Task<ProcessResult> Verify() => ProcessRunner.RunAsync(["msbuild", "GuiInterop.fpasproj", "-nologo", "-t:FpasVerifyPublish",
            "-p:FpasPublishDirectory=" + output, "-p:FpasPublishVerificationCertificate=" + publicKey],
            workingDirectory: project, environment: environment);

        // Reuse the owned directory so the transitions also exercise removal of stale sidecars.
        foreach (string mode in new[] { "framework-dependent", "apphost", "self-contained", "single-file", "aot" })
        {
            var publish = await ProcessRunner.RunAsync([compiler, "publish", "GuiInterop.fpasproj", "--mode", mode,
                "--runtime", rid, "--configuration", configuration, "--output", output,
                "--certificate", pfx, "--certificate-password-env", passwordVariable],
                workingDirectory: project, environment: publishEnvironment, timeLimit: TimeSpan.FromMinutes(8));
            Assert.True(publish.ExitCode == 0, mode + ": " + publish.Output + publish.Error);
            string entry = CheckPublishedLayout(output, mode, sharedName, sharedHash, notices);
            var verified = await Verify();
            Assert.True(verified.ExitCode == 0, mode + ": " + verified.Output + verified.Error);
            if (mode == "framework-dependent")
            {
                await CheckTamperRejection(Path.Combine(output, sharedName), Verify);
                await CheckTamperRejection(Path.Combine(output, "licenses", "cui", Path.GetFileName(notices[0])), Verify);
            }
            await RunPublishedGui(directory, output, mode, entry, sharedName, sharedHash, environment);
        }
    }

    private static string CheckPublishedLayout(string output, string mode, string sharedName, string sharedHash, string[] notices)
    {
        string name = OperatingSystem.IsWindows() ? "GuiInterop.exe" : "GuiInterop";
        bool directoryMode = mode is "framework-dependent" or "apphost" or "self-contained";
        string coreclr = OperatingSystem.IsWindows() ? "coreclr.dll" : "libcoreclr.so";
        Assert.Equal(mode != "framework-dependent", File.Exists(Path.Combine(output, name)));
        Assert.Equal(mode == "self-contained", File.Exists(Path.Combine(output, coreclr)));
        Assert.Equal(directoryMode, File.Exists(Path.Combine(output, "GuiInterop.dll")));
        Assert.Equal(directoryMode ? new[] { Path.Combine(output, sharedName) } : [],
            Directory.GetFiles(output, sharedName, SearchOption.AllDirectories));

        using var manifest = JsonDocument.Parse(File.ReadAllBytes(Path.Combine(output, ".fpas-publish.json")));
        var hashes = manifest.RootElement.GetProperty("files").EnumerateArray().ToDictionary(
            file => file.GetProperty("path").GetString()!, file => file.GetProperty("sha256").GetString());
        if (directoryMode)
        {
            Assert.Equal(sharedHash, FileHash(Path.Combine(output, sharedName)));
            Assert.Equal(sharedHash, hashes[sharedName]);
        }
        foreach (string notice in notices)
        {
            string relative = "licenses/cui/" + Path.GetFileName(notice);
            string hash = FileHash(notice);
            Assert.Equal(hash, FileHash(Path.Combine(output, relative)));
            Assert.Equal(hash, hashes[relative]);
        }
        string entry = mode == "framework-dependent" ? "GuiInterop.dll" : name;
        Assert.Equal(FileHash(Path.Combine(output, entry)), hashes[entry]);
        return entry;
    }

    private static async Task RunPublishedGui(TemporaryDirectory directory, string output, string mode,
        string entry, string sharedName, string sharedHash, IReadOnlyDictionary<string, string?> environment)
    {
        string executable = Path.Combine(output, entry);
        if (mode == "aot")
        {
            string dependencies = await NativeDependencies.Read(executable);
            Assert.DoesNotContain("fpas_cui", dependencies, StringComparison.OrdinalIgnoreCase);
            Assert.Contains(OperatingSystem.IsWindows() ? "opengl32.dll" : "libX11.so.6", dependencies, StringComparison.OrdinalIgnoreCase);
        }
        if (mode is "single-file" or "aot")
        {
            string delivery = Directory.CreateDirectory(directory.File(mode + " executable only")).FullName;
            string copy = Path.Combine(delivery, entry);
            File.Copy(executable, copy);
            if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(copy, File.GetUnixFileMode(executable));
            executable = copy;
        }
        string runDirectory = Directory.CreateDirectory(directory.File(mode + " run elsewhere")).FullName;
        string extraction = Directory.CreateDirectory(directory.File(mode + " extraction")).FullName;
        var runEnvironment = new Dictionary<string, string?>(environment)
        {
            ["DOTNET_BUNDLE_EXTRACT_BASE_DIR"] = extraction
        };
        if (mode is "self-contained" or "single-file" or "aot")
        {
            runEnvironment["PATH"] = "";
            runEnvironment["DOTNET_ROOT"] = directory.File("absent");
            runEnvironment["DOTNET_ROOT_X64"] = directory.File("absent");
            runEnvironment["DOTNET_HOST_PATH"] = directory.File("absent");
        }
        foreach (string renderer in new[] { "software", "opengl" })
        {
            string capture = Path.Combine(runDirectory, renderer + ".png");
            var run = mode == "framework-dependent"
                ? await ProcessRunner.RunAsync([executable, renderer, capture], workingDirectory: runDirectory, environment: runEnvironment)
                : await ProcessRunner.RunExecutableAsync(executable, [renderer, capture], workingDirectory: runDirectory, environment: runEnvironment);
            CheckGuiRun(run, renderer, capture);
        }
        if (mode == "single-file")
            Assert.Equal(sharedHash, FileHash(Assert.Single(Directory.GetFiles(extraction, sharedName, SearchOption.AllDirectories))));
        else
            Assert.Empty(Directory.EnumerateFileSystemEntries(extraction));
    }

    private static async Task CheckTamperRejection(string file, Func<Task<ProcessResult>> verify)
    {
        byte[] original = File.ReadAllBytes(file);
        try
        {
            File.AppendAllText(file, "tampering");
            var altered = await verify();
            Assert.NotEqual(0, altered.ExitCode);
            Assert.Contains("FP5011", altered.Output + altered.Error);
        }
        finally
        {
            File.WriteAllBytes(file, original);
        }
    }

    private static string FileHash(string path)
    {
        using var stream = File.OpenRead(path);
        return Convert.ToHexString(SHA256.HashData(stream));
    }

    private static async Task CheckPackagedBuild(string project, string configuration, string rid,
        IReadOnlyDictionary<string, string?> environment, TemporaryDirectory directory)
    {
        var build = await ProcessRunner.RunAsync(["build", "GuiInterop.fpasproj", "--configuration", configuration, "-warnaserror"],
            workingDirectory: project, environment: environment, timeLimit: TimeSpan.FromMinutes(3));
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        string output = Path.Combine(project, "bin", configuration, "net10.0");
        string shared = OperatingSystem.IsWindows() ? "fpas_cui.dll" : "libfpas_cui.so";
        Assert.True(File.Exists(Path.Combine(output, shared)));
        Assert.Equal(6, Directory.GetFiles(Path.Combine(output, "licenses", "cui"), "LICENSE-*.txt").Length);
        string work = Directory.CreateDirectory(directory.File("managed run elsewhere")).FullName;
        string capture = Path.Combine(work, "software.png");
        var run = await ProcessRunner.RunAsync([Path.Combine(output, "GuiInterop.dll"), "software", capture],
            workingDirectory: work, environment: environment);
        CheckGuiRun(run, "software", capture);

        string other = rid == "win-x64" ? "linux-x64" : "win-x64";
        foreach (var (invalidRid, expected) in new[]
        {
            ("osx-x64", "does not support runtime identifier 'osx-x64'"),
            (other, $"package is missing the native asset for '{other}'")
        })
        {
            var validation = await ProcessRunner.RunAsync(["msbuild", "GuiInterop.fpasproj", "-t:AssignTargetPaths",
                "-p:RuntimeIdentifier=" + invalidRid], workingDirectory: project, environment: environment);
            Assert.NotEqual(0, validation.ExitCode);
            Assert.Contains(expected, validation.Output + validation.Error);
        }
    }

}
