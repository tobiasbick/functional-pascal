using Fpas.IntegrationTests.Documentation.Support;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Text.Json;

namespace Fpas.IntegrationTests.Documentation;

[Collection("Documentation examples")]
public sealed class DocExamplesBehaviorTests(DocExamplesFixture checker)
{
    private const string Program = "program Example; begin end; end program;";
    private const string Body = "pascal fragment context=body";
    private static string Fence(string info, string source) => "```" + info + "\n" + source + "\n```\n";

    public static IEnumerable<object[]> Scenarios()
    {
        yield return ["program", Fence("pascal", Program), true, ""];
        yield return ["fragment", Fence(Body, "const Value: int := 7;"), true, ""];
        yield return ["expected-error", Fence(Body + " error=FP3002", "const Value: int := 'wrong';"), true, ""];
        yield return ["run", Fence(Body + " run stdout=empty", "if 6 * 7 <> 42 then raise new Sys.InvalidOperationException('wrong'); end if;"), true, ""];
        yield return ["exception", Fence(Body + " run exception=System.InvalidOperationException", "raise new Sys.InvalidOperationException('expected');"), true, ""];
        yield return ["compile-only", Fence(Body, "raise new Sys.InvalidOperationException('must not run');"), true, ""];
        yield return ["runtime-failure", Fence(Body + " run stdout=empty", "raise new Sys.InvalidOperationException('unexpected');"), false, "Execution did not satisfy"];
        yield return ["wrong-exception", Fence(Body + " run exception=System.ArgumentException", "raise new Sys.InvalidOperationException('unexpected');"), false, "Execution did not satisfy"];
        yield return ["missing-exception", Fence(Body + " run exception=System.ArgumentException", "const Value: int := 1;"), false, "Execution did not satisfy"];
        yield return ["timeout", Fence(Body + " run stdout=empty", "while true do Sys.Threading.Thread.Sleep(20); end while;"), false, "timeout"];
        yield return ["missing-expectation", Fence("pascal run", Program), false, "requires exactly one"];
        yield return ["unit-run", Fence("pascal run stdout=empty", "unit Example; end unit;"), false, "unit cannot"];
        yield return ["canonical", Fence("Pascal", Program), false, "canonical pascal"];
        yield return ["unknown-option", Fence("pascal skip", Program), false, "Unknown fence option"];
        yield return ["repeated-option", Fence("pascal run run stdout=empty", Program), false, "Repeated fence option"];
        yield return ["duplicate-id", Fence("pascal id=same", Program) + Fence("pascal id=same", Program), false, "Duplicate example id"];
        yield return ["invalid-id", Fence("pascal id=bad.name", Program), false, "Invalid example"];
        yield return ["invalid-code", Fence("pascal error=FPabcd", Program), false, "Expected error=FP"];
        yield return ["unclosed", "~~~pascal\n" + Program, false, "Unclosed Markdown fence"];
        yield return ["tilde-crlf", ("~~~~pascal\tfragment context=body\nconst Value: int := 7;\n~~~~\n").Replace("\n", "\r\n"), true, ""];
        yield return ["wrong-diagnostic", Fence(Body + " error=FP3002", "const Value: int := Missing;"), false, "FP3001"];
        yield return ["missing-error", Fence("pascal error=FP3002", Program), false, "Expected FP3002"];
        yield return ["broken-example", "# Heading\n\n" + Fence(Body, "const Value: int := 'wrong';"), false, "FP3002"];
        yield return ["fragment-without-context", Fence("pascal fragment", "const Value: int := 1;"), false, "requires context"];
        yield return ["unknown-context", Fence("pascal fragment context=absent", "const Value: int := 1;"), false, "Missing context"];
        yield return ["broken-context", Fence("pascal fragment context=broken error=FP3002", "const Value: int := 1;"), false, "from the example itself"];
        yield return ["bad-slot-count", Fence("pascal fragment context=slots", "const Value: int := 1;"), false, "exactly one"];
        yield return ["include", Fence("pascal fragment context=included", "const Answer: int := Value * 6;"), true, ""];
        yield return ["unknown-include", Fence("pascal fragment context=unknown-include", "const Value: int := 1;"), false, "Unknown context example"];
        yield return ["include-error", Fence("pascal fragment context=bad-include error=FP3002", "const Answer: int := 1;"), false, "from the example itself"];
        yield return ["unsafe-path", Fence("pascal file=../escape.fpas", Program), false, "Invalid example output path"];
        yield return ["duplicate-file", Fence("pascal project=demo file=same.fpas", Program) + Fence("pascal project=demo file=SAME.fpas", "unit Other; end unit;"), false, "Duplicate project output file"];
        yield return ["two-programs", Fence("pascal project=demo file=one.fpas", Program) + Fence("pascal project=demo file=two.fpas", "program Other; begin end; end program;"), false, "exactly one program"];
        yield return ["companion-error", Fence("pascal project=demo file=main.fpas error=FP3002", Program) + Fence("pascal project=demo file=values.fpas error=FP3002", "unit Values; const Broken: int := 'wrong'; end unit;"), false, "from the example itself"];
        yield return ["missing-manifest", Fence("pascal project=demo manifest=Missing.fpasproj", Program), false, "Missing documented project"];
        yield return ["broken-library-manifest", Fence("xml project=demo file=Demo.fpasproj", "<Project") + Fence("pascal project=demo manifest=Demo.fpasproj", "unit Values; end unit;"), false, "Documented project failed"];
        yield return ["output-limit", Fence(Body + " run stdout=empty", "Sys.Console.Write(new Sys.String(#120, 600000));"), false, "output exceeded"];
        yield return ["no-examples", Fence("text", "not source"), false, "No Pascal examples"];
        yield return ["unsupported-project", Fence("yaml project=demo file=fixture.yaml", "value: 1") + Fence("pascal", Program), false, "Unsupported project fence"];
        yield return ["stdout", Fence("pascal project=answer run stdout=expected.txt", "program Answer; uses System.Console as Console; begin Console.WriteLine(42); end; end program;") + Fence("text project=answer file=expected.txt", "42"), true, ""];
        yield return ["wrong-stdout", Fence("pascal project=answer run stdout=expected.txt", "program Answer; uses System.Console as Console; begin Console.WriteLine(7); end; end program;") + Fence("text project=answer file=expected.txt", "42"), false, "Execution did not satisfy"];
    }

    [Fact]
    public async Task Existing_output_directories_are_refused_without_touching_their_contents()
    {
        using var root = new TemporaryDirectory(); Prepare(root);
        File.WriteAllText(root.File("example.md"), Fence("pascal", Program));
        Directory.CreateDirectory(root.File("output")); File.WriteAllText(root.File("output/keep.txt"), "original");
        var result = await checker.Run(root.Path, "--document", "example.md", "--output", root.File("output"), "--json");
        Assert.Equal(1, result.ExitCode); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Contains("must be new", report.RootElement.GetProperty("error").GetString());
        Assert.Equal("original", File.ReadAllText(root.File("output/keep.txt")));
        Assert.Single(Directory.GetFileSystemEntries(root.File("output")));
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Timeout_and_console_cancellation_join_the_example_and_its_descendants(bool cancel)
    {
        using var root = new TemporaryDirectory(); Prepare(root);
        var ready = root.File("ready.pid"); var childReady = root.File("child.pid");
        static string Literal(string value) => value.Replace("'", "''", StringComparison.Ordinal);
        var source = $$"""
            program Waiting;
            uses System as S, System.IO as IO, System.Diagnostics as P, System.Threading as T;
            procedure Ready(Path: string);
            begin IO.File.WriteAllText(Path + '.tmp', S.Environment.ProcessId.ToString()); IO.File.Move(Path + '.tmp', Path); end procedure;
            begin
              if args.Length = 0 then
                const Start: P.ProcessStartInfo := new P.ProcessStartInfo('{{Literal(TestEnvironment.DotnetHost)}}');
                Start.UseShellExecute := false;
                Start.ArgumentList.Add(S.Environment.GetCommandLineArgs()[0]); Start.ArgumentList.Add('child');
                discard P.Process.Start(Start);
                while not IO.File.Exists('{{Literal(childReady)}}') do T.Thread.Sleep(20); end while;
                Ready('{{Literal(ready)}}');
              else Ready('{{Literal(childReady)}}');
              end if;
              while true do T.Thread.Sleep(20); end while;
            end;
            end program;
            """;
        File.WriteAllText(root.File("waiting.md"), Fence("pascal run stdout=empty", source));
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        await using var process = InterruptibleProcess.Start(TestEnvironment.DotnetHost,
            checker.Arguments(root.Path, "--document", "waiting.md", "--timeout", cancel ? "30" : "5", "--json"), root.Path);
        process.Input.Dispose();
        using var stdout = new StreamReader(process.Output); using var stderr = new StreamReader(process.Error);
        var output = stdout.ReadToEndAsync(deadline.Token); var error = stderr.ReadToEndAsync(deadline.Token);
        var pids = new List<int>();
        try
        {
            while (!File.Exists(ready))
            {
                if (process.HasExited) Assert.Fail("Example did not become ready: " + await output + await error);
                await Task.Delay(20, deadline.Token);
            }
            pids.Add(int.Parse(File.ReadAllText(ready), System.Globalization.CultureInfo.InvariantCulture));
            pids.Add(int.Parse(File.ReadAllText(childReady), System.Globalization.CultureInfo.InvariantCulture));
            if (cancel) await process.Interrupt();
            await process.WaitForExit(deadline.Token);
            Assert.Equal(cancel ? 130 : 1, process.ExitCode); Assert.Empty(await error);
            using var report = JsonDocument.Parse(await output);
            Assert.True(report.RootElement.GetProperty("failed").GetInt32() > 0);
            foreach (var pid in pids) Assert.Throws<ArgumentException>(() => Process.GetProcessById(pid));
        }
        finally
        {
            foreach (var pid in pids)
            {
                try { using var target = Process.GetProcessById(pid); if (!target.HasExited) target.Kill(true); }
                catch (ArgumentException) { }
            }
        }
    }

    [Theory]
    [MemberData(nameof(Scenarios))]
    public async Task Markdown_context_compiler_and_execution_failures_cannot_silently_pass(string name, string markdown, bool accepted, string expected)
    {
        using var root = new TemporaryDirectory();
        Prepare(root);
        var document = root.File(name + ".md"); File.WriteAllText(document, markdown);
        var result = await checker.Run(root.Path, "--document", document, "--output", root.File("output"), "--timeout", "5", "--json");
        Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.True(accepted == (result.ExitCode == 0), name + ": " + result.Output);
        Assert.Equal(accepted, report.RootElement.GetProperty("failed").GetInt32() == 0);
        if (expected.Length > 0) Assert.Contains(expected, result.Output, StringComparison.OrdinalIgnoreCase);
        if (name == "broken-example") Assert.Equal(4, Assert.Single(report.RootElement.GetProperty("results").EnumerateArray()).GetProperty("line").GetInt32());
        if (name == "include-error") Assert.Contains("definitions.md", Assert.Single(report.RootElement.GetProperty("results").EnumerateArray()).GetProperty("document").GetString());
    }

    [Fact]
    public async Task Documented_manifests_build_without_rewriting_their_sources_and_units_compile_independently()
    {
        using var root = new TemporaryDirectory(); Prepare(root);
        var project = $"<Project Sdk=\"Fpas.NET.Sdk/{checker.SdkVersion}\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType></PropertyGroup></Project>";
        File.WriteAllText(root.File("NuGet.Config"), new System.Xml.Linq.XDocument(new System.Xml.Linq.XElement("configuration", new System.Xml.Linq.XElement("packageSources",
            new System.Xml.Linq.XElement("clear"), new System.Xml.Linq.XElement("add", new System.Xml.Linq.XAttribute("key", "fpas"), new System.Xml.Linq.XAttribute("value", checker.Feed))))).ToString());
        const string unit = "unit Values; public const Answer: int := 42; end unit;";
        const string program = "program Answer; uses Values as V, System.Console as Console; begin Console.WriteLine(V.Answer); end; end program;";
        File.WriteAllText(root.File("project.md"), Fence("xml project=demo file=Demo.fpasproj", project) +
            Fence("pascal project=demo file=values.fpas manifest=Demo.fpasproj", unit) +
            Fence("pascal project=demo file=main.fpas manifest=Demo.fpasproj run stdout=expected.txt", program) + Fence("text project=demo file=expected.txt", "42"));
        var result = await checker.Run(root.Path, "--document", "project.md", "--json", "--timeout", "30");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal(2, report.RootElement.GetProperty("total").GetInt32());
        var outcomes = report.RootElement.GetProperty("results").EnumerateArray().ToArray();
        Assert.Single(outcomes, item => item.GetProperty("library").GetBoolean());
        var executed = Assert.Single(outcomes, item => item.GetProperty("run").GetString() == "passed");
        var generated = Path.GetDirectoryName(executed.GetProperty("assembly").GetString())!;
        Assert.Equal(program + "\n", File.ReadAllText(Path.Combine(generated, "main.fpas")));
        Assert.Equal(project + "\n", File.ReadAllText(Path.Combine(generated, "Demo.fpasproj")));
    }

    private static void Prepare(TemporaryDirectory root)
    {
        void Write(string name, string source)
        {
            var path = root.File("apps/doc-examples/contexts/" + name + ".fpas.txt");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!); File.WriteAllText(path, source);
        }
        const string header = "program Example;\nuses System as Sys;\n";
        Write("body", header + "begin\n// @example\nend; end program;\n");
        Write("broken", header + "const Broken: int := 'wrong';\nbegin\n// @example\nend; end program;\n");
        Write("slots", header + "begin\n// @example\n// @example\nend; end program;\n");
        Write("included", header + "// @include definitions.md#value\nbegin\n// @example\nend; end program;\n");
        Write("unknown-include", header + "// @include definitions.md#absent\nbegin\n// @example\nend; end program;\n");
        Write("bad-include", header + "// @include definitions.md#broken\nbegin\n// @example\nend; end program;\n");
        File.WriteAllText(root.File("definitions.md"), Fence(Body + " id=value", "const Value: int := 7;") + Fence(Body + " id=broken", "const Broken: int := 'wrong';"));
    }
}
