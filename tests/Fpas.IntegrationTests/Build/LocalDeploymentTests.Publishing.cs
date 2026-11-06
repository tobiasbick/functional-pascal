using System.Security.Cryptography;
using System.Security.Cryptography.Pkcs;
using System.Security.Cryptography.X509Certificates;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class LocalDeploymentTests
{
    private static async Task CheckPublishing(string consumer, string launcher, string feed, string version, IReadOnlyDictionary<string, string?> environment)
    {
        var root = Path.Combine(Path.GetDirectoryName(consumer)!, "publish-consumer");
        Directory.CreateDirectory(root);
        var project = Path.Combine(root, "App.fpasproj");
        File.WriteAllText(project, $"<Project Sdk=\"Fpas.NET.Sdk/{version}\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType></PropertyGroup></Project>");
        File.WriteAllText(Path.Combine(root, "main.fpas"), "program App; uses System.Console as Console; begin Console.WriteLine(42); end; end program;");
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "fpas"), new XAttribute("value", feed)),
            new XElement("add", new XAttribute("key", "nuget"), new XAttribute("value", "https://api.nuget.org/v3/index.json")))))
            .Save(Path.Combine(root, "NuGet.Config"));
        using var rsa = RSA.Create(2048);
        using var certificate = new CertificateRequest("CN=FPAS deployment test", rsa, HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1)
            .CreateSelfSigned(DateTimeOffset.UtcNow.AddMinutes(-1), DateTimeOffset.UtcNow.AddDays(1));
        var password = Guid.NewGuid().ToString("N"); var pfx = Path.Combine(root, "publisher.pfx");
        File.WriteAllBytes(pfx, certificate.Export(X509ContentType.Pfx, password));
        var variables = environment.ToDictionary(pair => pair.Key, pair => pair.Value);
        variables["FPAS_TEST_PUBLISH_PASSWORD"] = password;
        var output = Path.Combine(Path.GetDirectoryName(root)!, "published-app");
        var published = await ProcessRunner.RunExecutableAsync(launcher, ["publish", project, "--output", output, "--mode", "apphost",
            "--certificate", pfx, "--certificate-password-env", "FPAS_TEST_PUBLISH_PASSWORD", "--report", "json"],
            workingDirectory: root, environment: variables, timeLimit: TimeSpan.FromMinutes(3));
        Assert.True(published.ExitCode == 0, published.Output + published.Error);
        Assert.DoesNotContain(password, published.Output + published.Error);
        var signature = new SignedCms(); signature.Decode(File.ReadAllBytes(Path.Combine(output, ".fpas-signature.p7s")));
        signature.CheckSignature(verifySignatureOnly: true);
        Assert.Equal(certificate.RawData, Assert.Single(signature.SignerInfos.Cast<SignerInfo>()).Certificate!.RawData);
        Assert.Empty(Directory.GetFiles(output, "*.pfx", SearchOption.AllDirectories));
        var ran = await ProcessRunner.RunExecutableAsync(Path.Combine(output, OperatingSystem.IsWindows() ? "App.exe" : "App"), [],
            workingDirectory: output, environment: environment);
        Assert.True(ran.ExitCode == 0, ran.Output + ran.Error); Assert.Equal("42\n", ran.Output.ReplaceLineEndings("\n"));
    }
}
