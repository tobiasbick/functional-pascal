using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class PartialAnalysisTests
{
    private static string[] References => [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location];
    private static AnalysisResult Analyze(params CompilationSource[] sources) => FpasCompiler.Analyze(new(sources, "Partial", References, CompilationTarget.Library));

    [Theory]
    [InlineData("return Missing;")]
    [InlineData("return Target(;")]
    [InlineData("const Value: := 1; return Value;")]
    [InlineData("const Value: int := 1 return Value;")]
    public void A_broken_body_preserves_checked_declarations_and_independent_bodies(string broken)
    {
        var text = "unit Sample; function Target(): int; begin return 42; end function; " +
            "function Broken(): int; begin " + broken + " end function; " +
            "function Healthy(): int; begin return Target(); end function; end unit;";
        var input = new CompilationSource("input.fpas", text); var result = Analyze(input);
        Assert.False(result.Success); Assert.False(result.IsComplete); Assert.Equal("partial", result.Availability);
        var snapshot = Assert.IsType<SemanticSnapshot>(result.Snapshot);
        Assert.Contains(snapshot.Symbols, symbol => symbol.Name == "Broken" && symbol.Kind == "routine");
        var call = text.LastIndexOf("Target()", StringComparison.Ordinal);
        var occurrence = Assert.IsType<SymbolOccurrence>(snapshot.OccurrenceAt(input.Path, call)); Assert.Equal("Target", snapshot.Symbol(occurrence.Symbol).Name);
        var failed = text.IndexOf(broken, StringComparison.Ordinal);
        Assert.False(snapshot.Coverage.AvailableAt(input.Path, failed)); Assert.True(snapshot.Coverage.AvailableAt(input.Path, call));
        Assert.DoesNotContain(snapshot.Occurrences, fact => fact.Span.Offset >= failed && fact.Span.Offset < failed + broken.Length);
        Assert.DoesNotContain(snapshot.Symbols, symbol => symbol.Kind == "local" && symbol.Name == "Value");
        AssertConsistent(snapshot);
        var compiled = FpasCompiler.Compile(new([input], "Partial", References, CompilationTarget.Library));
        Assert.False(compiled.Success); Assert.Null(compiled.Program);
    }

    [Theory]
    [InlineData("function Broken(Value: ): int; begin return 1; end function;")]
    [InlineData("function Broken(Value: MissingType): int; begin return 1; end function;")]
    [InlineData("type Broken = record Value: ; end record;")]
    [InlineData("const Value: int := Missing; function Read(): int; begin return 1; end function;")]
    public void Invalid_declarations_remove_their_scope_and_consumers_but_preserve_independent_units(string declaration)
    {
        var result = Analyze(new("broken.fpas", "unit Broken; public " + declaration + " end unit;"),
            new("consumer.fpas", "unit Consumer; uses Broken as B; function Run(): int; begin return 1; end function; end unit;"),
            new("healthy.fpas", "unit Healthy; function Answer(): int; begin return 42; end function; end unit;"));
        Assert.Equal("partial", result.Availability); Assert.False(result.Success);
        var snapshot = Assert.IsType<SemanticSnapshot>(result.Snapshot);
        Assert.Contains(snapshot.Symbols, symbol => symbol.Name == "Answer");
        Assert.DoesNotContain(snapshot.Symbols, symbol => symbol.Name == "Run");
        Assert.False(snapshot.Coverage.AvailableAt("broken.fpas", 1)); Assert.False(snapshot.Coverage.AvailableAt("consumer.fpas", 1));
        Assert.True(snapshot.Coverage.AvailableAt("healthy.fpas", 1)); AssertConsistent(snapshot);
    }

    [Theory]
    [InlineData("unit Sample; function Healthy(): int; begin return 42; end function;")]
    [InlineData("unit Sample; function Healthy(): int; begin return 42; end function; function Broken(): int; begin return 1;")]
    [InlineData("unit Sample; function Healthy(): int; begin return 42; end function; function Broken(): int;")]
    [InlineData("unit Sample; function Healthy(): int; begin return 42; end function; function Broken(): int; begin return 1; end unit;")]
    [InlineData("unit Sample; function Healthy(): int; begin return 42; end function; function Broken(): int; begin return 1; public function Later(): int; begin return 7; end function; end unit;")]
    public void Missing_closers_retain_checked_prefixes_without_making_the_program_compilable(string text)
    {
        var input = new CompilationSource("input.fpas", text); var result = Analyze(input);
        Assert.False(result.Success); Assert.False(result.IsComplete); Assert.NotEmpty(result.Diagnostics);
        var snapshot = Assert.IsType<SemanticSnapshot>(result.Snapshot);
        Assert.Contains(snapshot.Symbols, symbol => symbol.Name == "Healthy");
        Assert.Contains(snapshot.Bodies, body => body.Symbol is { } id && snapshot.Symbol(id).Name == "Healthy");
        Assert.False(FpasCompiler.Compile(new([input], "Partial", References, CompilationTarget.Library)).Success); AssertConsistent(snapshot);
    }

    private static void AssertConsistent(SemanticSnapshot snapshot)
    {
        var symbols = snapshot.Symbols.Select(symbol => symbol.Id).ToHashSet(); var bodies = snapshot.Bodies.Select(body => body.Id).ToHashSet();
        foreach (var occurrence in snapshot.Occurrences) { Assert.Contains(occurrence.Symbol, symbols); Assert.True(snapshot.Coverage.Available(occurrence.Span)); }
        foreach (var expression in snapshot.Expressions) Assert.True(snapshot.Coverage.Available(expression.Span));
        foreach (var body in snapshot.Bodies) { Assert.True(snapshot.Coverage.Available(body.Span)); if (body.Parent is { } parent) Assert.Contains(parent, bodies); }
        foreach (var operation in snapshot.Operations) { Assert.Contains(operation.Body, bodies); if (operation.Target is { } target) Assert.Contains(target, symbols); }
        foreach (var symbol in snapshot.Symbols)
        { if (symbol.Context?.Owner is { } owner) Assert.Contains(owner, symbols); foreach (var surface in symbol.Context?.SurfaceSymbols ?? []) Assert.Contains(surface, symbols); }
        foreach (var type in snapshot.Types) if (type.Parameter is { } parameter) Assert.Contains(parameter, symbols);
    }

    [Fact]
    public void Coverage_keeps_nested_errors_inside_their_scope_and_rejects_unknown_documents()
    {
        var document = new SourceDocument("input.fpas", new string('x', 100), CancellationToken.None);
        var coverage = new SemanticCoverage([document], [new(document.Span(10, 50), "unavailable", "body_error"), new(document.Span(30, 0), "unavailable", "syntax_error")]);
        Assert.False(coverage.AvailableAt(document.Path, 30)); Assert.True(coverage.AvailableAt(document.Path, 60));
        Assert.False(coverage.AvailableAt("unknown.fpas", 0)); Assert.False(coverage.AvailableAt(document.Path, 101));
        Assert.Single(coverage.Regions, region => region.State == "unavailable");
    }

    [Fact]
    public void Failed_outer_bodies_remove_nested_routines_lambdas_and_generic_local_facts()
    {
        const string source = "unit Sample; function Target(): int; begin return 42; end function; " +
            "function Broken(): int; begin function Inner<T>(Value: T): T; begin return Value; end function; " +
            "const Callback: function(): int := function(): int => Target(); discard Inner<int>(1); return Missing; end function; " +
            "function Healthy(): int; begin return Target(); end function; end unit;";
        var result = Analyze(new CompilationSource("input.fpas", source)); Assert.False(result.Success); var snapshot = Assert.IsType<SemanticSnapshot>(result.Snapshot);
        Assert.DoesNotContain(snapshot.Symbols, symbol => symbol.Name is "Inner" or "Callback" or "Value");
        Assert.DoesNotContain(snapshot.Bodies, body => body.Kind == "lambda"); AssertConsistent(snapshot);
    }

    [Fact]
    public void A_broken_program_entry_preserves_checked_routines_without_publishing_entry_facts()
    {
        const string source = "program Sample; function Healthy(): int; begin return 42; end function; begin discard Healthy(); discard Missing; end; end program;";
        var result = FpasCompiler.Analyze(new([new("input.fpas", source)], "Partial", References)); var snapshot = Assert.IsType<SemanticSnapshot>(result.Snapshot);
        Assert.False(result.Success); Assert.Contains(snapshot.Symbols, symbol => symbol.Name == "Healthy");
        Assert.DoesNotContain(snapshot.Bodies, body => body.Kind == "program_entry"); Assert.Empty(snapshot.Operations); AssertConsistent(snapshot);
    }
}
