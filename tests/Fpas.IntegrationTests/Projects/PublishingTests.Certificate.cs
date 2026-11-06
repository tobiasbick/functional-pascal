using System.Security.Cryptography;
using System.Security.Cryptography.Pkcs;
using System.Security.Cryptography.X509Certificates;
using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

public sealed partial class PublishingTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Native_delivery_signature_pins_certificate_covers_files_and_rejects_tampering(bool foreignHost)
    {
        using var directory = new TemporaryDirectory(); var project = await Create(directory);
        var verificationProject = project;
        if (foreignHost)
        {
            var library = System.Xml.Linq.XDocument.Load(project);
            library.Root!.Element("PropertyGroup")!.Element("OutputType")!.Value = "Library";
            library.Root.Element("PropertyGroup")!.Add(new System.Xml.Linq.XElement("AssemblyName", "PublishedLibrary")); library.Save(project);
            File.WriteAllText(directory.File("App/main.fpas"), "unit PublishApi; public function Read(): int; begin return 42; end function; end unit;");
            System.IO.Directory.CreateDirectory(directory.File("Host"));
            project = directory.File("Host/Host.csproj");
            File.WriteAllText(project, "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>App</AssemblyName></PropertyGroup><ItemGroup><ProjectReference Include=\"../App/App.fpasproj\"/></ItemGroup></Project>");
            File.WriteAllText(directory.File("Host/Program.cs"), "System.Console.WriteLine(PublishApi.Read());");
            File.Copy(directory.File("App/NuGet.Config"), directory.File("Host/NuGet.Config"));
        }
        using var key = RSA.Create(2048);
        var request = new CertificateRequest("CN=FPAS publishing test", key, HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1);
        using var certificate = request.CreateSelfSigned(DateTimeOffset.UtcNow.AddMinutes(-1), DateTimeOffset.UtcNow.AddDays(1));
        var pfx = Path.Combine(Path.GetDirectoryName(project)!, "publisher.pfx"); var publicKey = directory.File("publisher.cer");
        var password = Guid.NewGuid().ToString("N");
        File.WriteAllBytes(pfx, certificate.Export(X509ContentType.Pfx, password));
        File.WriteAllBytes(publicKey, certificate.Export(X509ContentType.Cert));
        var xml = System.Xml.Linq.XDocument.Load(project);
        xml.Root!.Add(new System.Xml.Linq.XElement("ItemGroup", new System.Xml.Linq.XElement("None", new System.Xml.Linq.XAttribute("Update", "publisher.pfx"), new System.Xml.Linq.XAttribute("CopyToPublishDirectory", "Always")))); xml.Save(project);
        var environment = new Dictionary<string, string?> { ["FPAS_TEST_PFX_PASSWORD"] = password };
        var output = directory.File("delivery");
        var published = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "publish", project, "--output", output,
            "--certificate", pfx, "--certificate-password-env", "FPAS_TEST_PFX_PASSWORD", "--report", "json"],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(8));
        Assert.True(published.ExitCode == 0, published.Output + published.Error);
        Assert.DoesNotContain(password, published.Output + published.Error);
        var signature = Path.Combine(output, ".fpas-signature.p7s");
        var signed = new SignedCms(); signed.Decode(File.ReadAllBytes(signature)); signed.CheckSignature(verifySignatureOnly: true);
        Assert.Equal(certificate.RawData, Assert.Single(signed.SignerInfos.Cast<SignerInfo>()).Certificate!.RawData);
        using var manifest = JsonDocument.Parse(signed.ContentInfo.Content);
        var entries = manifest.RootElement.GetProperty("files").EnumerateArray().ToArray();
        Assert.NotEmpty(entries);
        foreach (var entry in entries)
        {
            var path = Path.Combine(output, entry.GetProperty("path").GetString()!);
            Assert.Equal(entry.GetProperty("sha256").GetString(), Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path))));
        }
        Assert.Empty(Directory.GetFiles(output, "*.pfx", SearchOption.AllDirectories));
        Task<ProcessResult> Verify(string expected) => ProcessRunner.RunAsync(["msbuild", verificationProject, "-nologo", "-t:FpasVerifyPublish",
            "-p:FpasPublishDirectory=" + output, "-p:FpasPublishVerificationCertificate=" + expected], workingDirectory: directory.Path);
        var verified = await Verify(publicKey); Assert.True(verified.ExitCode == 0, verified.Output + verified.Error);
        using var otherKey = RSA.Create(2048);
        using var other = new CertificateRequest("CN=Other test publisher", otherKey, HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1)
            .CreateSelfSigned(DateTimeOffset.UtcNow.AddMinutes(-1), DateTimeOffset.UtcNow.AddDays(1));
        var unexpected = directory.File("other.cer"); File.WriteAllBytes(unexpected, other.Export(X509ContentType.Cert));
        Assert.NotEqual(0, (await Verify(unexpected)).ExitCode);
        var executable = Path.Combine(output, OperatingSystem.IsWindows() ? "App.exe" : "App");
        var original = File.ReadAllBytes(executable); File.AppendAllText(executable, "tampering");
        var altered = await Verify(publicKey); Assert.NotEqual(0, altered.ExitCode); Assert.Contains("FP5011", altered.Output + altered.Error);
        File.WriteAllBytes(executable, original);
        File.WriteAllText(pfx, "invalid certificate");
        var failed = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "publish", project, "--output", output,
            "--certificate", pfx, "--report", "json"], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(8));
        Assert.Equal(1, failed.ExitCode); Assert.Equal(original, File.ReadAllBytes(executable));
    }
}
