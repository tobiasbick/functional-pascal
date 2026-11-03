using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Text.RegularExpressions;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed class LanguageSpecificationTests
{
    private static string Root => Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification");
    private static readonly Regex Fences = new(@"(?ms)^```pascal\r?\n(.*?)^```", RegexOptions.CultureInvariant);
    private static readonly Regex Header = new(@"(?m)^(?:async )?(program|unit) ([A-Za-z_][A-Za-z_0-9.]*)\s*;", RegexOptions.CultureInvariant);

    public static IEnumerable<object[]> CompleteExamples()
    {
        foreach (var path in Directory.GetFiles(Root, "*.md", SearchOption.AllDirectories).Order(StringComparer.Ordinal))
            foreach (var (match, index) in Fences.Matches(File.ReadAllText(path)).Cast<Match>().Select((match, index) => (match, index)))
                if (Header.Match(match.Groups[1].Value.Trim()) is { Success: true } header)
                    yield return [Path.GetRelativePath(Root, path), index, header.Groups[2].Value, header.Groups[1].Value == "unit"];
    }

    [Theory]
    [MemberData(nameof(CompleteExamples))]
    public async Task Complete_documented_sources_compile_and_execute(string document, int fence, string name, bool library)
    {
        using var directory = new TemporaryDirectory();
        var source = Fences.Matches(File.ReadAllText(Path.Combine(Root, document)))[fence].Groups[1].Value;
        Assert.EndsWith(library ? "end unit;" : "end program;", source.Trim());
        var sourcePath = directory.File(name + ".fpas");
        File.WriteAllText(sourcePath, source);
        var sources = new List<string> { sourcePath };
        if (name == "MovePoint")
        {
            var geometry = Fences.Matches(File.ReadAllText(Path.Combine(Root, "program-structure", "README.md")))
                .Cast<Match>().Single(match => match.Groups[1].Value.StartsWith("unit Game.Geometry;", StringComparison.Ordinal));
            var dependency = directory.File("Geometry.fpas");
            File.WriteAllText(dependency, geometry.Groups[1].Value); sources.Add(dependency);
        }
        var output = directory.File(name + ".dll");
        var compile = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", .. sources,
            "--output", output, "--target", library ? "library" : "exe", "--reference-dir", TestEnvironment.ReferenceDirectory,
            "--generate-documentation", "--diagnostics", "json"]);
        Assert.True(compile.ExitCode == 0, document + ": " + compile.Output + compile.Error);
        Assert.True(File.Exists(Path.ChangeExtension(output, ".pdb")));
        Assert.Equal(name, XDocument.Load(Path.ChangeExtension(output, ".xml")).Root!.Element("assembly")!.Element("name")!.Value);
        if (library)
        {
            Assert.False(File.Exists(Path.ChangeExtension(output, ".runtimeconfig.json")));
            return;
        }
        File.Copy(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasCliPath"))!, "Fpas.Core.dll"), directory.File("Fpas.Core.dll"));
        File.WriteAllText(directory.File("world.json"), "{\"world\":\"test\"}");
        var environment = new Dictionary<string, string> { ["DOTNET_SYSTEM_GLOBALIZATION_INVARIANT"] = "1", ["FPAS_PLAYER"] = "Explorer" };
        var result = name == "DownloadPage" ? await RunWithRejectedProxy(output, directory.Path, environment)
            : await ProcessRunner.RunAsync([output], workingDirectory: directory.Path, environment: environment);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Empty(result.Error);
        var lines = result.Output.Replace("\r\n", "\n", StringComparison.Ordinal).TrimEnd('\n').Split('\n');
        string[]? expected = name switch
        {
            "Hello" when document == "examples.md" => ["Hello, Mara", "20", "4000000000", "1.5", "12.50", "M", "True"],
            "Hello" => ["Hello, CLR"],
            "Functions" => ["Functions first", "42", "1", "2"],
            "Shapes" => ["1", "4", "12"],
            "Collections" => ["9", "Mara", "Keeper", "10", "42"],
            "NullableText" => ["Explorer", "8", "3", "A"],
            "MoveOutcomes" => ["The edge of the map"],
            "ReadWorld" => ["{\"world\":\"test\"}", "Read attempt finished"],
            "Interop" => ["3", "{\"player_name\":\"Mara\",\"Score\":10}"],
            "TypeRules" => ["8080", "Branch", "-1", "An unnamed CLR enum value"],
            "Identities" => ["42", "Mara", "False"],
            "RecordMembers" => ["2", "3", "Mara", "1", "1"],
            "MovePoint" => ["4"],
            "AsyncHello" => ["Ready"],
            _ => null
        };
        if (expected is not null) Assert.Equal(expected, lines);
        else switch (name)
        {
            case "Events": Assert.All(lines.Where(line => line.Length != 0), line => Assert.Equal("Tick", line)); break;
            case "DownloadPage": Assert.Contains("502", result.Output); break;
            case "Contracts": Assert.Equal(["5", "0"], lines[..2]); Assert.Contains("requires", result.Output); break;
            case "Ranges": Assert.Equal(["60", "1"], lines[..2]); Assert.Contains("range", result.Output, StringComparison.OrdinalIgnoreCase); break;
            case "Mandelbrot": Assert.Equal(24, lines.Length); Assert.All(lines, line => Assert.Equal(80, line.Length)); break;
            default: Assert.Fail("Add an execution assertion for the complete documented example " + name); break;
        }
        if (name == "ReadWorld")
        {
            File.Delete(directory.File("world.json"));
            var missing = await ProcessRunner.RunAsync([output], workingDirectory: directory.Path, environment: environment);
            Assert.Equal(0, missing.ExitCode); Assert.Empty(missing.Error);
            Assert.Contains("world.json", missing.Output); Assert.EndsWith("Read attempt finished" + Environment.NewLine, missing.Output);
        }
    }

    private static async Task<ProcessResult> RunWithRejectedProxy(string output, string directory, Dictionary<string, string> environment)
    {
        using var proxy = new TcpListener(IPAddress.Loopback, 0);
        proxy.Start();
        var address = "http://127.0.0.1:" + ((IPEndPoint)proxy.LocalEndpoint).Port;
        environment["HTTPS_PROXY"] = environment["https_proxy"] = address;
        environment["NO_PROXY"] = environment["no_proxy"] = "";
        var run = ProcessRunner.RunAsync([output], workingDirectory: directory, environment: environment);
        using var connection = await proxy.AcceptTcpClientAsync().WaitAsync(TimeSpan.FromSeconds(10));
        using var stream = connection.GetStream();
        using var reader = new StreamReader(stream, leaveOpen: true);
        while (await reader.ReadLineAsync().WaitAsync(TimeSpan.FromSeconds(10)) is { Length: > 0 }) { }
        await stream.WriteAsync(Encoding.ASCII.GetBytes("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"));
        return await run;
    }
}
