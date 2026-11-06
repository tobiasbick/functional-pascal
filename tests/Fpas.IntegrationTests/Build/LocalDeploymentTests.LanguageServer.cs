using Fpas.IntegrationTests.LanguageServer;
using Fpas.Tests.Support;
using System.Security.Cryptography;
using System.Text.Json;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.Build;

public sealed partial class LocalDeploymentTests
{
    private static async Task CheckLanguageServer(string consumer, string launcher, string sdkCache, IReadOnlyDictionary<string, string?> environment)
    {
        var source = Path.Combine(consumer, "Probe.fpas"); var text = File.ReadAllText(source);
        var binaries = new[] { "Fpas.LanguageServer.dll", "Fpas.SdkResolver.dll", "OmniSharp.Extensions.LanguageProtocol.dll", "StreamJsonRpc.dll", "Newtonsoft.Json.dll" };
        foreach (var name in binaries)
            Assert.Equal(File.ReadAllBytes(Path.Combine(Path.GetDirectoryName(launcher)!, name)), File.ReadAllBytes(Path.Combine(sdkCache, "tools", "compiler", name)));
        foreach (var executable in new[] { true, false })
        {
            var entry = executable ? launcher : Path.Combine(sdkCache, "tools", "compiler", "Fpas.Cli.dll");
            var inspected = executable
                ? await ProcessRunner.RunExecutableAsync(entry, ["env", "--json"], workingDirectory: consumer, environment: environment)
                : await ProcessRunner.RunAsync([entry, "env", "--json"], workingDirectory: consumer, environment: environment);
            Assert.True(inspected.ExitCode == 0, inspected.Output + inspected.Error); Assert.Empty(inspected.Error);
            using var env = JsonDocument.Parse(inspected.Output);
            Assert.Equal("available", env.RootElement.GetProperty("languageServer").GetProperty("status").GetString());
            var before = Fingerprints(consumer);
            await using var client = new LspProbe(entry, executable, consumer, environment);
            var initialized = Result(await client.InitializeResponse(Path.Combine(consumer, "Probe.fpasproj")));
            Assert.True(initialized.GetProperty("capabilities").GetProperty("definitionProvider").GetBoolean());
            await client.Notify("initialized", new { }); await client.Open(source, text, 7);
            var diagnostics = await client.Diagnostics(source, 7); Assert.Empty(diagnostics.GetProperty("params").GetProperty("diagnostics").EnumerateArray());
            var location = Result(await client.Request("textDocument/definition", new { textDocument = Document(source), position = Position(text, "Read():") }));
            Assert.Equal(new Uri(source).AbsoluteUri, location.GetProperty("uri").GetString());
            Assert.Single(Result(await client.Request("workspace/symbol", new { query = "Read" })).EnumerateArray());
            var rename = Result(await client.Request("textDocument/rename", new { textDocument = Document(source), position = Position(text, "Read():"), newName = "Fetch" }));
            Assert.Equal(7, Assert.Single(rename.GetProperty("documentChanges").EnumerateArray()).GetProperty("textDocument").GetProperty("version").GetInt32());
            await client.Stop(); Assert.Equal(before, Fingerprints(consumer));
        }

        static string[] Fingerprints(string root) => Directory.GetFiles(root, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
            .Select(path => path + ":" + File.GetLastWriteTimeUtc(path).Ticks + ":" + Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path)))).ToArray();
    }
}
