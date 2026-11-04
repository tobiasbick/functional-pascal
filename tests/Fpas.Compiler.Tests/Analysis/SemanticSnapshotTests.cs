using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;
using static Fpas.Compiler.Tests.Analysis.Support.SemanticAnalysis;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class SemanticSnapshotTests
{
    [Theory]
    [InlineData("\n")]
    [InlineData("\r\n")]
    [InlineData("\r")]
    public void Exact_UTF16_spans_preserve_escaped_names_tabs_Unicode_and_innermost_expressions(string newline)
    {
        var source = string.Join(newline, "// Grüße 🙂", "program Coordinates;", "begin", "\tconst @type: int := (1 + 20) * 2;", "\tdiscard @type + 3;", "end;", "end program;");
        var snapshot = Analyze(source);
        var document = Assert.Single(snapshot.Documents);
        Assert.Equal(source, document.Text);
        var declaration = Assert.Single(snapshot.Symbols, symbol => symbol.Name == "type");
        Assert.Equal("@type", Slice(source, declaration.Declaration!));
        Assert.Equal(4, declaration.Declaration!.Line); Assert.Equal(8, declaration.Declaration.Column);
        var use = source.LastIndexOf("@type", StringComparison.Ordinal);
        Assert.Equal(declaration.Id, snapshot.OccurrenceAt("input.fpas", use + 1)!.Symbol);
        Assert.Null(snapshot.OccurrenceAt("input.fpas", use + 5));
        Assert.Equal(use, document.Offset(5, 10));
        var plus = source.IndexOf("+", StringComparison.Ordinal);
        var expression = snapshot.TypeAt("input.fpas", plus)!;
        Assert.Equal("1 + 20", Slice(source, expression.Span));
        Assert.Equal("int", snapshot.Types.Single(type => type.Id == expression.Type).Display);
        Assert.Throws<ArgumentOutOfRangeException>(() => document.Offset(0, 1));
        Assert.Throws<ArgumentOutOfRangeException>(() => document.Offset(4, 500));
    }

    [Fact]
    public void Shadowing_and_bound_overloads_have_distinct_symbols_and_signatures()
    {
        const string source = """
            program Bindings;
            function Pick(Value: int): int; begin return Value; end function;
            function Pick(Value: string): string; begin return Value; end function;
            begin
              const Value: int := Pick(21);
              begin const Value: string := Pick('text'); discard Value; end;
              discard Value;
            end;
            end program;
            """;
        var snapshot = Analyze(source);
        var methods = snapshot.Symbols.Where(symbol => symbol.Kind == "routine" && symbol.Name == "Pick").ToArray();
        Assert.Equal(2, methods.Length); Assert.NotEqual(methods[0].Id, methods[1].Id);
        var first = snapshot.Symbol(snapshot.OccurrenceAt("input.fpas", source.IndexOf("Pick(21)", StringComparison.Ordinal))!.Symbol);
        var second = snapshot.Symbol(snapshot.OccurrenceAt("input.fpas", source.IndexOf("Pick('text')", StringComparison.Ordinal))!.Symbol);
        Assert.NotEqual(first.Id, second.Id);
        Assert.Equal("int", snapshot.Types.Single(type => type.Id == first.Callable!.Result).Display);
        Assert.Equal("string", snapshot.Types.Single(type => type.Id == second.Callable!.Result).Display);
        var inner = snapshot.OccurrenceAt("input.fpas", source.IndexOf("discard Value;", StringComparison.Ordinal) + 8)!;
        var outer = snapshot.OccurrenceAt("input.fpas", source.LastIndexOf("discard Value;", StringComparison.Ordinal) + 8)!;
        Assert.NotEqual(inner.Symbol, outer.Symbol);
        Assert.Equal(6, snapshot.Symbol(inner.Symbol).Declaration!.Line);
        Assert.Equal(5, snapshot.Symbol(outer.Symbol).Declaration!.Line);
        Assert.Equal("callable", snapshot.Types.Single(type => type.Id == snapshot.TypeAt("input.fpas", source.IndexOf("Pick(21)", StringComparison.Ordinal))!.Type).Kind);
    }

    [Fact]
    public void Captures_keep_the_original_binding_and_flow_types_remain_distinct_from_declared_types()
    {
        const string source = """
            program Captures;
            begin
              var Text: string? := null;
              procedure SetValue(); begin Text := 'value'; end procedure;
              SetValue();
              if Text <> null then discard Text.Length; end if;
            end;
            end program;
            """;
        var snapshot = Analyze(source);
        var declaration = Assert.Single(snapshot.Symbols, symbol => symbol.Kind == "local" && symbol.Name == "Text");
        var written = snapshot.OccurrenceAt("input.fpas", source.IndexOf("Text :=", StringComparison.Ordinal))!;
        Assert.Equal(declaration.Id, written.Symbol); Assert.Equal("write", written.Role);
        var read = source.IndexOf("Text.Length", StringComparison.Ordinal);
        Assert.Equal(declaration.Id, snapshot.OccurrenceAt("input.fpas", read)!.Symbol);
        var selected = snapshot.TypeAt("input.fpas", read)!;
        Assert.False(snapshot.Types.Single(type => type.Id == selected.Type).Nullable);
        Assert.True(snapshot.Types.Single(type => type.Id == selected.DeclaredType).Nullable);
        Assert.DoesNotContain(snapshot.Symbols, symbol => symbol.Name.Contains("Closure", StringComparison.Ordinal));
    }

    [Fact]
    public void Metadata_is_detached_and_symbol_handles_cannot_cross_changed_snapshots()
    {
        using var directory = new TemporaryDirectory();
        var library = FpasCompiler.Compile(new([new("library.fpas", "unit Library; public function Read(): int; begin return 42; end function; end unit;")],
            "Dependency", References, CompilationTarget.Library));
        Assert.True(library.Success);
        var path = directory.File("Dependency.dll"); File.WriteAllBytes(path, library.Program!.AssemblyImage);
        const string source = "program Consumer; uses Library as L; begin discard L.Read(); end; end program;";
        var result = FpasCompiler.Analyze(new([new("input.fpas", source)], "Input", [.. References, path]));
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        var snapshot = result.Snapshot!;
        File.Delete(path);
        var symbol = snapshot.Symbol(snapshot.OccurrenceAt("input.fpas", source.IndexOf("Read()", StringComparison.Ordinal))!.Symbol);
        Assert.Null(symbol.Declaration); Assert.Equal("Dependency", symbol.Metadata!.Assembly);
        Assert.Equal("Library", symbol.Metadata.Type); Assert.Equal("Read", symbol.Metadata.Name);
        var changed = Analyze("program Consumer; begin end; end program;");
        Assert.NotEqual(snapshot.Id, changed.Id);
        Assert.Throws<ArgumentException>(() => changed.Symbol(symbol.Id));
        Assert.Throws<NotSupportedException>(() => ((IList<SemanticSymbol>)snapshot.Symbols).Clear());
    }

    private static string Slice(string text, SourceSpan span) => text.Substring(span.Offset, span.Length);
}
