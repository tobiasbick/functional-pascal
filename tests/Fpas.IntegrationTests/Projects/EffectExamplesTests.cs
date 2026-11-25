// [Documentation](../../../docs/pascal/libraries/effects.md#io-examples)
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Xml.Linq;
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Documentation;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

/// <summary>Builds and runs the documented effect examples with the packaged SDK.</summary>
/// <param name="sdk">The matching SDK package for the example projects.</param>
public sealed class EffectExamplesTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Checks an HTTP response body, a failed status and cancellation against a local listener.</summary>
    [Fact]
    public async Task Http_effect_handles_body_status_and_cancellation()
    {
        using var workspace = new TemporaryDirectory();
        var program = await Build(workspace, "http-effect", "HttpEffect");
        await Exchange("200 OK", "Response from the local server\n", 0, "Response from the local server");
        await Exchange("404 Not Found", "Missing\n", 1, "HTTP status 404");
        await Exchange(null, "", 1, "Request canceled");

        async Task Exchange(string? status, string body, int exitCode, string output)
        {
            using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(15));
            using var listener = new TcpListener(IPAddress.Loopback, 0);
            listener.Start();
            var address = $"http://127.0.0.1:{((IPEndPoint)listener.LocalEndpoint).Port}/text";
            var server = Serve();
            try
            {
                var run = await ProcessRunner.RunAsync([program, address], workingDirectory: workspace.Path,
                    environment: new Dictionary<string, string?> { ["NO_PROXY"] = "127.0.0.1", ["no_proxy"] = "127.0.0.1" },
                    timeLimit: TimeSpan.FromSeconds(12));
                Assert.True(run.ExitCode == exitCode, run.Output + run.Error);
                Assert.Equal(output + Environment.NewLine, run.Output);
                Assert.Empty(run.Error);
                if (status is not null) await server;
            }
            finally
            {
                deadline.Cancel();
                try { await server; }
                catch (OperationCanceledException) when (deadline.IsCancellationRequested) { }
            }

            async Task Serve()
            {
                using var connection = await listener.AcceptTcpClientAsync(deadline.Token);
                using var stream = connection.GetStream();
                using var reader = new StreamReader(stream, Encoding.ASCII, leaveOpen: true);
                Assert.Equal("GET /text HTTP/1.1", await reader.ReadLineAsync(deadline.Token));
                while (await reader.ReadLineAsync(deadline.Token) is { Length: > 0 }) { }
                if (status is null)
                {
                    await Task.Delay(Timeout.InfiniteTimeSpan, deadline.Token);
                    return;
                }

                var bytes = Encoding.UTF8.GetBytes(body);
                var headers = $"HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {bytes.Length}\r\nConnection: close\r\n\r\n";
                await stream.WriteAsync(Encoding.ASCII.GetBytes(headers), deadline.Token);
                await stream.WriteAsync(bytes, deadline.Token);
            }
        }
    }

    /// <summary>Checks that invalid URI input receives the usage error before a request starts.</summary>
    [Fact]
    public async Task Http_effect_reports_invalid_addresses_as_usage_errors()
    {
        using var workspace = new TemporaryDirectory();
        var program = await Build(workspace, "http-effect", "HttpEffect");
        foreach (var address in new[] { "invalid address", "relative/path", "ftp://example.com/file" })
        {
            var run = await ProcessRunner.RunAsync([program, address], workingDirectory: workspace.Path);
            Assert.Equal(2, run.ExitCode);
            Assert.Equal("Pass one absolute HTTP URL." + Environment.NewLine, run.Output);
            Assert.Empty(run.Error);
        }
    }

    /// <summary>Checks the typed success and missing-row results of a parameterized SQLite query.</summary>
    [Fact]
    public async Task Sqlite_effect_reads_a_title_and_reports_a_missing_row()
    {
        using var workspace = new TemporaryDirectory();
        var program = await Build(workspace, "sqlite-effect", "SqliteEffect", restoreProvider: true);
        var run = await ProcessRunner.RunAsync([program], workingDirectory: workspace.Path);
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Functional Pascal\nMissing book 2\n", run.Output.ReplaceLineEndings("\n"));
        Assert.Empty(run.Error);
    }

    private async Task<string> Build(TemporaryDirectory workspace, string example, string name, bool restoreProvider = false)
    {
        var repository = DocumentationLinks.Repository;
        var source = Path.Combine(repository, "examples", "io", example);
        var project = workspace.File(name + ".fpasproj");
        var file = XDocument.Load(Path.Combine(source, name + ".fpasproj"));
        file.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + sdk.Version);
        file.Save(project);
        File.Copy(Path.Combine(source, "main.fpas"), workspace.File("main.fpas"));
        sdk.WriteFeedConfig(workspace.Path);
        if (restoreProvider)
        {
            File.Copy(Path.Combine(repository, "Directory.Packages.props"), workspace.File("Directory.Packages.props"));
            var configPath = workspace.File("NuGet.Config");
            var config = XDocument.Load(configPath);
            config.Root!.Element("packageSources")!.Add(new XElement("add",
                new XAttribute("key", "nuget.org"), new XAttribute("value", "https://api.nuget.org/v3/index.json")));
            config.Save(configPath);
        }

        var build = await sdk.Run(workspace, "build", project, "-warnaserror");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        return workspace.File($"bin/Debug/net10.0/{name}.dll");
    }
}
