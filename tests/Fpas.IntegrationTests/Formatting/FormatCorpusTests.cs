using Fpas.Compiler;
using Fpas.Compiler.Formatting;
using Fpas.IntegrationTests.Documentation;
using Fpas.IntegrationTests.Support;

namespace Fpas.IntegrationTests.Formatting;

public sealed class FormatCorpusTests
{
    public static IEnumerable<object[]> DocumentedSources() => MarkdownExamples.Read(Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification"))
        .Select(example => new object[] { example.Document + ":" + example.Fence, example.Source });

    [Theory]
    [MemberData(nameof(DocumentedSources))]
    public void Complete_documentation_sources_preserve_tokens_documentation_and_are_idempotent(string name, string input)
        => Check(name, input);

    [Fact]
    public void All_copied_complete_source_fixtures_are_formattable_and_idempotent()
    {
        var root = Path.Combine(AppContext.BaseDirectory, "Fixtures");
        var contexts = Path.Combine(root, "doc-examples", "contexts") + Path.DirectorySeparatorChar;
        var files = Directory.GetFiles(root, "*.fpas", SearchOption.AllDirectories)
            .Where(file => !file.StartsWith(contexts, StringComparison.Ordinal)).ToArray();
        Assert.True(files.Length > 20);
        foreach (var file in files) Check(file, File.ReadAllText(file));
    }

    [Fact]
    public void Canonical_formatter_documentation_is_already_formatted()
    {
        var path = Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification", "tools", "fmt-style.md");
        var example = Assert.Single(MarkdownExamples.Read(path));
        Assert.Equal(example.Source.Replace("\r\n", "\n", StringComparison.Ordinal), Check(example.Name, example.Source));
    }

    [Fact]
    public async Task Formatting_preserves_captures_cleanup_case_expressions_and_numeric_results()
    {
        const string input = """
            program RoundTrip;
            uses System as Sys,System.Console as Console;
            function Make():function():int;
            begin var Count:int:=1;return function():int begin Count:=Count+1;return Count;end function;end function;
            begin
            const Next:function():int:=Make();Console.WriteLine(Next());Console.WriteLine(Next());
            const Value:int:=case 3 of when 1..2:10;when 3:20;else 30;end case;
            Console.WriteLine(Value);
            try raise new Sys.InvalidOperationException('test');catch E:Sys.Exception do Console.WriteLine(E.Message);finally Console.WriteLine('cleanup');end try;
            Console.WriteLine($FF+1_000);Console.WriteLine(0.1M+0.2M);
            end;end program;
            """;
        var formatted = Check("roundtrip.fpas", input);
        var before = await ProgramExecution.Run(input); var after = await ProgramExecution.Run(formatted);
        Assert.Equal(0, before.ExitCode); Assert.Equal(before, after);
        Assert.Contains("cleanup", after.Output);
    }

    [Fact]
    public async Task Formatting_record_updates_preserves_copying_and_field_evaluation_order()
    {
        const string input = """
            program UpdateOrder;
            uses System.Console as Console;
            type Pair = record X:int;Y:int;end record;
            var Count:int:=0;
            function Next():int;begin Count:=Count+1;return Count;end function;
            begin
            const Original:Pair:=new Pair(10,20);
            const Changed:Pair:=Original with Y:=Next();X:=Next();end with;
            Console.WriteLine(Original.X);Console.WriteLine(Original.Y);
            Console.WriteLine(Changed.X);Console.WriteLine(Changed.Y);Console.WriteLine(Count);
            end;end program;
            """;
        var formatted = Check("updates.fpas", input);
        var before = await ProgramExecution.Run(input); var after = await ProgramExecution.Run(formatted);
        Assert.Equal(0, before.ExitCode); Assert.Equal("10\n20\n2\n1\n2\n", before.Output);
        Assert.Equal(before, after);
    }

    private static string Check(string name, string input)
    {
        var first = FpasFormatter.Format(new(name, input));
        Assert.True(first.Success, name + ": " + string.Join('\n', first.Diagnostics));
        var second = FpasFormatter.Format(new(name, first.Text!));
        Assert.True(second.Success, name + ": " + string.Join('\n', second.Diagnostics));
        Assert.Equal(first.Text, second.Text);
        return first.Text!;
    }
}
