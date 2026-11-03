using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed class LanguageSpecificationTests
{
    private static string Root => Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification");
    private static IReadOnlyList<MarkdownExample> Examples() => [.. MarkdownExamples.Read(Root),
        .. MarkdownExamples.Read(Path.Combine(AppContext.BaseDirectory, "Fixtures", "repository", "README.md"), "repository/")];
    public static IEnumerable<object[]> CompleteExamples() => Examples()
        .Select(example => new object[] { example.Document, example.Fence, example.Name, example.Library });

    [Theory]
    [MemberData(nameof(CompleteExamples))]
    public async Task Complete_documented_sources_compile_and_execute(string document, int fence, string name, bool library)
    {
        using var directory = new TemporaryDirectory();
        var examples = Examples();
        var example = examples.Single(item => item.Document == document && item.Fence == fence);
        Assert.EndsWith(library ? "end unit;" : "end program;", example.Source.Trim());
        var sources = new List<string>();
        foreach (var companion in MarkdownExamples.Sources(example, examples))
        {
            var path = directory.File(companion.Name + ".fpas");
            File.WriteAllText(path, companion.Source); sources.Add(path);
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
        var environment = new Dictionary<string, string?> { ["DOTNET_SYSTEM_GLOBALIZATION_INVARIANT"] = "1", ["FPAS_PLAYER"] = "Explorer",
            ["FPAS_RECIPE_ATTEMPTS"] = "5" };
        var result = name is "DownloadPage" or "FetchUtf8" ? await RunWithRejectedProxy(output, directory.Path, environment)
            : await ProcessRunner.RunAsync([output], workingDirectory: directory.Path, environment: environment);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Empty(result.Error);
        var lines = result.Output.Replace("\r\n", "\n", StringComparison.Ordinal).TrimEnd('\n').Split('\n');
        string[]? expected = name switch
        {
            "PrimitiveValues" => ["Hello, Mara", "20", "4000000000", "1.5", "12.50", "M", "True"],
            "Hello" when document == "repository/README.md" => ["Hello, World!"],
            "Hello" => ["Hello, CLR"],
            "Fibonacci" => ["Fibonacci sequence:", "0", "1", "1", "2", "3", "5", "8", "13", "21", "34"],
            "PatternMatching" => ["Stop"],
            "HigherOrderFunctions" => ["20"],
            "GenericArithmetic" => ["42"],
            "NullableExample" => ["Found: 15", "Not found"],
            "ParseExample" => ["Doubled: 42", "Input error"],
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
            "AgentStart" => ["42"],
            "AgentBasics" => ["Basics verified"],
            "AgentTypes" => ["Types verified"],
            "AgentUnits" => ["Units verified"],
            "AgentFunctions" => ["Functions verified"],
            "AgentPatterns" => ["Patterns verified"],
            "AgentInterop" => ["Interop verified"],
            "AgentEvents" => ["Events verified"],
            "AgentAsync" => ["Async verified"],
            "HelloWorld" => ["Hello, world!"],
            "TextAndRegex" => ["room#", "3"],
            "LoopValues" => ["7"],
            "ResourceLifetime" => ["Resource released"],
            "EnvironmentConfiguration" => ["5"],
            "TimeAndDuration" => ["2030-04-05T10:30:00.0000000+00:00", "01:30:00"],
            "NumericUtilities" => ["12.34", "12.35", "43"],
            "FetchUtf8" => ["HTTP request failed"],
            _ => null
        };
        if (expected is not null) Assert.Equal(expected, lines);
        else switch (name)
        {
            case "Events": Assert.All(lines.Where(line => line.Length != 0), line => Assert.Equal("Tick", line)); break;
            case "DownloadPage": Assert.Contains("502", result.Output); break;
            case "RandomValues":
                Assert.Equal(4, lines.Length);
                var random = new Random(1729);
                for (var index = 0; index < 3; index++) Assert.Equal(random.Next(1, 7).ToString(System.Globalization.CultureInfo.InvariantCulture), lines[index]);
                Assert.Equal("Random values verified", lines[3]);
                break;
            case "CaptureVersion":
                Assert.Equal(2, lines.Length);
                Assert.Equal("0", lines[0]);
                Assert.Matches(@"^\d+\.\d+\.\d+(?:[-+].+)?$", lines[1]);
                break;
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
        if (name == "EnvironmentConfiguration")
        {
            foreach (var (value, expectedOutput) in new (string?, string?)[]
                     { (null, "3"), ("1", "1"), ("10", "10"), (" +7 ", "7"), ("", null), ("invalid", null),
                         ("0", null), ("11", null), ("2147483648", null) })
            {
                environment["FPAS_RECIPE_ATTEMPTS"] = value;
                var configured = await ProcessRunner.RunAsync([output], workingDirectory: directory.Path, environment: environment);
                Assert.Equal(expectedOutput is null ? 2 : 0, configured.ExitCode);
                Assert.Equal(expectedOutput is null ? "" : expectedOutput + Environment.NewLine, configured.Output);
                Assert.Equal(expectedOutput is null ? "Invalid FPAS_RECIPE_ATTEMPTS" + Environment.NewLine : "", configured.Error);
            }
        }
    }

    private static async Task<ProcessResult> RunWithRejectedProxy(string output, string directory, Dictionary<string, string?> environment)
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
