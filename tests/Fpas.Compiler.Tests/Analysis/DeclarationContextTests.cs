using Fpas.Compiler.Analysis;
using static Fpas.Compiler.Tests.Analysis.Support.SemanticAnalysis;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class DeclarationContextTests
{
    [Fact]
    public void Declaration_fragments_do_not_absorb_neighbouring_items_or_implementation_scopes()
    {
        const string source = """
            program Families;
            uses System as S;
            type Number = int;
            type Key = distinct int;
            type Code = enum One; Two = 2; end enum;
            type IValue = interface Name: string; function Read(Value: int): int; end interface;
            type Choice = union None; Some(Item: int); end union;
            var Global: int := 1;
            begin
              for Index: int := 1 to 1 do discard Index; end for;
              try raise new S.Exception('example'); catch Error: S.Exception do discard Error.Message; end try;
              const Selected: Choice := Choice.Some(Item := 1);
              case Selected of
                when Choice.None: begin end;
                when Choice.Some(Item := const Payload: int): discard Payload;
              end case;
            end;
            end program;
            """;
        var snapshot = Analyze(source);
        foreach (var (name, expected, qualified) in new[]
        {
            ("Number", "type Number = int;", "Families.Number"),
            ("Key", "type Key = distinct int;", "Families.Key"),
            ("Two", "Two = 2;", "Families.Code.Two"),
            ("Name", "Name: string;", "Families.IValue.Name"),
            ("Read", "function Read(Value: int): int;", "Families.IValue.Read"),
            ("Item", "Item: int", "Families.Choice.Some.Item"),
            ("Some", "Some(Item: int);", "Families.Choice.Some"),
            ("Global", "var Global: int := 1;", "Families.Global"),
            ("Index", "Index: int", "Families.Index"),
            ("Error", "Error: S.Exception", "Families.Error"),
            ("Payload", "const Payload: int", "Families.Payload")
        })
        {
            var context = Assert.Single(snapshot.Symbols, symbol => symbol.Name == name && symbol.Declaration is not null).Context!;
            Assert.Equal(expected, source.Substring(context.Span.Offset, context.Span.Length));
            Assert.Equal(qualified, context.QualifiedName); Assert.Null(context.Body);
        }
        var contract = Assert.Single(snapshot.Symbols, symbol => symbol.Name == "IValue");
        Assert.Equal("Name: string; function Read(Value: int): int; ", source.Substring(contract.Context!.Body!.Offset, contract.Context.Body.Length));
    }

    [Theory]
    [InlineData("\n")]
    [InlineData("\r\n")]
    [InlineData("\r")]
    public void Declaration_context_preserves_ownership_exact_source_and_attached_documentation(string newline)
    {
        var source = string.Join(newline,
            "// Catalogue 🙂", "program Catalogue;", "uses System as Sys;",
            "type Box<T> = record", "  // Stored value", "  Value: T;",
            "  function Read(): T; begin return self.Value; end function;", "end record;",
            "// Outer documentation", "[Sys.ObsoleteAttribute('Example')]",
            "function Outer(Value: int): int;", "requires Value > 0;", "begin",
            "  const @type: int := Value;", "  function Inner(Input: int): int;",
            "  begin return Input + @type; end function;", "  return Inner(1);", "end function;",
            "begin discard Outer(1); end;", "end program;");
        var snapshot = Analyze(source);
        var outer = Symbol("Outer"); var inner = Symbol("Inner"); var local = Symbol("type");
        Assert.Equal("Catalogue.Outer.Inner", inner.Context!.QualifiedName);
        Assert.Equal(outer.Id, inner.Context.Owner); Assert.Equal(outer.Id, local.Context!.Owner);
        Assert.Equal("Catalogue.Outer.Inner.Input", Symbol("Input").Context!.QualifiedName);
        Assert.Equal("Catalogue.Box.T", Symbol("T").Context!.QualifiedName);
        Assert.Equal("private", outer.Context!.Accessibility);
        Assert.Equal("public", Symbol("Value", "property").Context!.Accessibility);
        Assert.Equal("const @type: int := Value;", Text(local.Context.Span));
        Assert.Equal("@type", Text(local.Declaration!));
        Assert.Equal("// Outer documentation", Text(outer.Context.Documentation!));
        Assert.StartsWith("[Sys.ObsoleteAttribute", Text(outer.Context.Span));
        Assert.EndsWith("end function;", Text(inner.Context.Span));
        Assert.EndsWith("return Inner(1);" + newline + "end function", Text(outer.Context.Body!));
        Assert.Null(Symbol("Value", "parameter").Context!.Body);
        Assert.Contains(outer.Context.SurfaceTypes, id => snapshot.Types.Single(type => type.Id == id).Display == "int");
        Assert.Equal(source, Text(Symbol("Catalogue").Context!.Span).Insert(0, "// Catalogue 🙂" + newline));

        SemanticSymbol Symbol(string name, string? kind = null) => Assert.Single(snapshot.Symbols, symbol => symbol.Name == name && (kind is null || symbol.Kind == kind));
        string Text(SourceSpan span) => source.Substring(span.Offset, span.Length);
    }
}
