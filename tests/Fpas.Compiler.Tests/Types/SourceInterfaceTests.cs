using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Types;

public sealed class SourceInterfaceTests
{
    [Fact]
    public void Former_interface_keyword_and_escaped_protocol_are_usable_as_identifiers()
    {
        var result = TestCompilation.Compile("""
            program KeywordNames;
            begin
              const interface: int := 1;
              const @protocol: int := interface;
              discard @protocol;
            end;
            end program;
            """);
        Assert.True(result.Success, string.Join('\n', result.Diagnostics));
    }

    [Theory]
    [InlineData("type I = interface end interface;", "end")]
    [InlineData("type I = protocol end interface;", "interface")]
    [InlineData("type protocol = int;", "protocol")]
    public void Protocol_syntax_rejects_old_declarations_and_unescaped_keyword_names(string declaration, string found)
    {
        var result = TestCompilation.Compile("unit Api; " + declaration + " end unit;", CompilationTarget.Library);
        Assert.False(result.Success);
        Assert.Equal(found, Assert.Single(result.Diagnostics).Expectation?.Found);
    }

    [Fact]
    public void Source_interfaces_emit_ordinary_abstract_CLR_contracts_and_documentation_deterministically()
    {
        const string source = """
            unit Api;
            uses System as Sys;
            // A named source.
            public type ISource<T> =
            where T: class;
            protocol extends Sys.IDisposable
              // The current value.
              Value: T;
              // Reads a value.
              function Read(): T;
              function Echo<U>(Value: U): U;
              where U: class;
            end protocol;
            public type Empty = protocol end protocol;
            end unit;
            """;
        var request = new CompilationRequest([new("api.fpas", source)], "Api",
            [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location], CompilationTarget.Library, EmitDocumentation: true);
        var result = FpasCompiler.Compile(request);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        Assert.Equal(result.Program!.AssemblyImage, FpasCompiler.Compile(request).Program!.AssemblyImage);
        using var pe = new PEReader(new MemoryStream(result.Program.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var contract = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "ISource`1");
        Assert.True(contract.Attributes.HasFlag(TypeAttributes.Interface | TypeAttributes.Abstract));
        Assert.Equal(TypeAttributes.NestedPublic, contract.Attributes & TypeAttributes.VisibilityMask);
        Assert.True(contract.BaseType.IsNil);
        Assert.Empty(contract.GetFields());
        Assert.Single(contract.GetProperties());
        Assert.Single(contract.GetInterfaceImplementations());
        var generic = reader.GetGenericParameter(Assert.Single(contract.GetGenericParameters()));
        Assert.Equal(GenericParameterAttributes.None, generic.Attributes & GenericParameterAttributes.VarianceMask);
        foreach (var method in contract.GetMethods().Select(reader.GetMethodDefinition))
        {
            Assert.True(method.Attributes.HasFlag(MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.Public));
            Assert.Equal(0, method.RelativeVirtualAddress);
        }
        var ids = XDocument.Parse(System.Text.Encoding.UTF8.GetString(result.Program.XmlDocumentation!))
            .Descendants("member").Select(member => (string)member.Attribute("name")!).ToArray();
        Assert.Equal(new[] { "M:Api.ISource`1.Read", "P:Api.ISource`1.Value", "T:Api.ISource`1" }, ids);
    }

    [Theory]
    [InlineData("type I = protocol extends I end protocol;", "FP3029")]
    [InlineData("type I<T> = protocol extends J<T> end protocol; type J<T> = protocol extends I<T> end protocol;", "FP3029")]
    [InlineData("type I = protocol extends Sys.String end protocol;", "FP3029")]
    [InlineData("type I = protocol end protocol; type J = protocol extends I? end protocol;", "FP3029")]
    [InlineData("type I = protocol end protocol; type J = protocol extends I, I end protocol;", "FP3029")]
    [InlineData("type I = protocol Name: string; function Name(): string; end protocol;", "FP3029")]
    [InlineData("type I = protocol Name: string; function get_Name(): string; end protocol;", "FP3029")]
    [InlineData("type I = protocol function F(): int; function F(): string; end protocol;", "FP3004")]
    [InlineData("type I = protocol function F(): int; end protocol; type R = record implements I end record;", "FP3029")]
    [InlineData("type I = protocol extends Sys.Collections.IList end protocol; type R = record implements I end record;", "FP3029")]
    [InlineData("type I = protocol procedure F(in Value: int); end protocol; type R = record implements I public procedure F(ref Value: int); begin end procedure; end record;", "FP3029")]
    [InlineData("type I = protocol function F<T>(Value: T): T; where T: class; end protocol; type R = record implements I public function F<T>(Value: T): T; begin return Value; end function; end record;", "FP3029")]
    [InlineData("type A = protocol function F(): int; end protocol; type B = protocol function F(): string; end protocol; type C = protocol extends A, B end protocol;", "FP3029")]
    [InlineData("type A = protocol Name: string; end protocol; type B = protocol function Name(): string; end protocol; type C = protocol extends A, B end protocol;", "FP3029")]
    [InlineData("type A = protocol function F<T>(Value: T): T; where T: class; end protocol; type B = protocol function F<U>(Value: U): U; where U: struct; end protocol; type C = protocol extends A, B end protocol;", "FP3029")]
    [InlineData("type A = protocol Value: string; end protocol; type B = protocol function get_Value(): string; end protocol; type C = protocol extends A, B end protocol;", "FP3029")]
    [InlineData("type I = protocol function F(Value: string?): string; end protocol; type R = record implements I public function F(Value: string): string; begin return Value; end function; end record;", "FP3029")]
    [InlineData("type I = protocol Value: string; end protocol; type R = record implements I public Value: string?; end record;", "FP3029")]
    [InlineData("type I = protocol procedure F(ref Value: string?); end protocol; type R = record implements I public procedure F(ref Value: string); begin end procedure; end record;", "FP3029")]
    [InlineData("type I = protocol [Sys.Diagnostics.CodeAnalysis.MaybeNullAttribute] Value: string; end protocol;", "FP3035")]
    [InlineData("type I = protocol [Sys.Diagnostics.CodeAnalysis.NotNullAttribute] Value: string?; end protocol;", "FP3035")]
    public void Invalid_interface_graphs_members_and_implementations_are_diagnosed(string declarations, string code)
    {
        var result = TestCompilation.Compile("program Invalid; uses System as Sys; " + declarations + " begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("private function F(): int;")]
    [InlineData("static function F(): int;")]
    [InlineData("async function F(): int;")]
    [InlineData("function F(): int; begin return 1; end function;")]
    [InlineData("Value: int := 1;")]
    [InlineData("function F(): int; requires true;")]
    public void Interface_declarations_reject_implementations_and_unsupported_member_forms(string member)
    {
        var result = TestCompilation.Compile("program Invalid; type I = protocol " + member + " end protocol; begin end; end program;");
        Assert.False(result.Success);
        Assert.StartsWith("FP20", Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("type I = protocol end protocol;", "discard new I();", "FP3003")]
    [InlineData("type I = protocol function Read(): int; end protocol; type R = record public function Read(): int; begin return 1; end function; end record;", "const Value: I := new R();", "FP3002")]
    [InlineData("type I<T> = where T: class; protocol end protocol;", "const Value: I<int>? := null;", "FP3018")]
    [InlineData("type I<T> = protocol end protocol; type R = record implements I<string> end record;", "const Value: I<object> := new R();", "FP3002")]
    [InlineData("type A = protocol Value: string; end protocol; type B = protocol Value: string?; end protocol; type C = protocol extends A, B end protocol; type R = record implements C public Value: string; end record;", "const Value: C := new R('x'); discard Value.Value;", "FP3003")]
    [InlineData("type A = protocol Value: int; end protocol; type B = protocol Value: int; end protocol; type C = protocol extends A, B end protocol; type R = record implements C public Value: int; end record;", "const Value: C := new R(1); discard Value.Value;", "FP3003")]
    [InlineData("type A = protocol function Read(): int; end protocol; type B = protocol function Read(): int; end protocol; type C = protocol extends A, B end protocol; type R = record implements C public function Read(): int; begin return 1; end function; end record;", "const Value: C := new R(); discard Value.Read();", "FP3003")]
    public void Interface_construction_constraints_nominality_and_ambiguous_uses_are_diagnosed(string declarations, string body, string code)
    {
        var result = TestCompilation.Compile("program Invalid; " + declarations + " begin " + body + " end; end program;");
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("type Hidden = protocol end protocol; public type Exposed = protocol extends Hidden end protocol;")]
    [InlineData("type Hidden = record end record; public type Exposed = protocol Value: Hidden; end protocol;")]
    [InlineData("type Hidden = protocol end protocol; public type Exposed<T> = where T: Hidden; protocol end protocol;")]
    public void Public_interfaces_cannot_expose_private_types(string declarations)
    {
        var result = TestCompilation.Compile("unit Invalid; " + declarations + " end unit;", CompilationTarget.Library);
        Assert.False(result.Success);
        Assert.Contains(Assert.Single(result.Diagnostics).Code, new[] { "FP3021", "FP3029" });
    }

    [Fact]
    public void Declaring_an_interface_does_not_require_record_implementation_support_for_its_imported_bases()
    {
        var result = TestCompilation.Compile("""
            unit Collections;
            uses System.Collections as C;
            public type IItems = protocol extends C.IList end protocol;
            end unit;
            """, CompilationTarget.Library);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
    }
}
