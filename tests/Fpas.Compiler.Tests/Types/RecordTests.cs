using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;
using static Fpas.Compiler.Tests.Support.TestCompilation;

namespace Fpas.Compiler.Tests.Types;

public sealed class RecordTests
{
    [Theory]
    [InlineData("const P: Point := new Point();", "FP3022")]
    [InlineData("const P: Point := new Point(X := 1, X := 2);", "FP3022")]
    [InlineData("const P: Point := new Point(Z := 1);", "FP3022")]
    [InlineData("const P: Point := new Point(X := 1, 2);", "FP3022")]
    [InlineData("const P: Point := new Point(1); P.X := 2;", "FP3009")]
    [InlineData("const P: Point := new Point(1) with Z := 2; end with;", "FP3022")]
    [InlineData("const P: Point := new Point(1) with X := 2; X := 3; end with;", "FP3022")]
    [InlineData("const P: string := 'x' with Length := 2; end with;", "FP3022")]
    [InlineData("const P: Point? := null; discard P with X := 1; end with;", "FP3013")]
    [InlineData("const P: Point := new Point(X := 'bad');", "FP3002")]
    public void Invalid_construction_and_updates_are_diagnosed(string body, string code)
        => Error("program Example; type Point = record public X: int; end record; begin " + body + " end; end program;", code);

    [Theory]
    [InlineData("type R = record public X: int; public X: int; end record;", "FP3021")]
    [InlineData("type R = record public Equals: int; end record;", "FP3021")]
    [InlineData("type int = record end record;", "FP3021")]
    [InlineData("type long = record end record;", "FP3021")]
    [InlineData("type R = record public X: int; public function get_X(): int; begin return 0; end function; end record;", "FP3021")]
    [InlineData("type R = record public X: int := self.X; end record;", "FP3001")]
    [InlineData("type R = record public X: int := X; end record;", "FP3001")]
    [InlineData("type R = record public static function Read(): int; begin return self.X; end function; end record;", "FP3001")]
    [InlineData("type R = record public function Read(): int; begin const self: int := 1; return self; end function; end record;", "FP3004")]
    public void Invalid_record_declarations_are_diagnosed(string declaration, string code)
        => Error("program Example; " + declaration + " begin end; end program;", code);

    [Theory]
    [InlineData("const R: Counter := new Counter(Value := 1);", "FP3011")]
    [InlineData("const R: Counter := Counter.Create(); discard R.Value;", "FP3011")]
    [InlineData("const R: Counter := Counter.Create(); discard R.Secret();", "FP3011")]
    [InlineData("discard Counter.Hidden();", "FP3011")]
    [InlineData("const R: Counter := Counter.Create() with Value := 2; end with;", "FP3011")]
    public void Private_members_are_restricted_to_the_record(string body, string code) => Error("""
        program Example;
        type Counter = record
          Value: int := 0;
          public static function Create(): Counter; begin return new Counter(); end function;
          function Secret(): int; begin return self.Value; end function;
          static function Hidden(): int; begin return 42; end function;
        end record;
        begin
        """ + " " + body + " end; end program;", code);

    [Fact]
    public void An_unmarked_record_method_cannot_implement_a_public_interface_method()
        => Error("""
            program Example;
            type IReader = interface function Read(): int; end interface;
            type Reader = record implements IReader
              function Read(): int; begin return 42; end function;
            end record;
            begin end;
            end program;
            """, "FP3029");

    [Fact]
    public void Public_signatures_cannot_expose_private_records()
        => Error("unit Model; type Hidden = record end record; public function Get(): Hidden; begin return new Hidden(); end function; end unit;", "FP3021", CompilationTarget.Library);

    [Theory]
    [InlineData("new Callback(Action := procedure() begin Text := null; end procedure)")]
    [InlineData("new Callback(procedure() begin end procedure) with Action := procedure() begin Text := null; end procedure; end with")]
    public void Record_expressions_participate_in_capture_and_loop_flow_analysis(string expression)
        => Error("program Example; type Callback = record public Action: procedure(); end record; begin var Text: string? := 'known'; for I: int in [1, 2] do discard Text.Length; const C: Callback := " +
            expression + "; C.Action(); end for; end; end program;", "FP3013");

    [Fact]
    public void Records_emit_sealed_nested_CLR_types_readonly_properties_and_equality_contracts()
    {
        var result = Compile("unit Models; public type Item = record public Name: string; public Count: int := 1; public Note: string? := null; end record; end unit;", CompilationTarget.Library);
        Assert.True(result.Success, string.Join(";", result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var type = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "Item");
        Assert.True(type.Attributes.HasFlag(TypeAttributes.Sealed));
        Assert.Equal(TypeAttributes.NestedPublic, type.Attributes & TypeAttributes.VisibilityMask);
        Assert.Equal("Models", reader.GetString(reader.GetTypeDefinition(type.GetDeclaringType()).Name));
        Assert.Single(type.GetInterfaceImplementations());
        Assert.Equal(["Name", "Count", "Note"], type.GetProperties().Select(handle => reader.GetString(reader.GetPropertyDefinition(handle).Name)).ToArray());
        Assert.All(type.GetProperties(), handle => Assert.True(reader.GetPropertyDefinition(handle).GetAccessors().Setter.IsNil));
        Assert.All(type.GetFields(), handle => Assert.True(reader.GetFieldDefinition(handle).Attributes.HasFlag(FieldAttributes.InitOnly)));
        var methods = type.GetMethods().Select(reader.GetMethodDefinition).ToArray();
        Assert.Equal(2, methods.Count(method => reader.GetString(method.Name) == "Equals"));
        Assert.Contains(methods, method => reader.GetString(method.Name) == "op_Equality" && method.Attributes.HasFlag(MethodAttributes.SpecialName));
        Assert.Contains(reader.AssemblyReferences, handle => reader.GetString(reader.GetAssemblyReference(handle).Name) == "Fpas.Core");
    }

    private static void Error(string source, string code, CompilationTarget target = CompilationTarget.Exe)
    {
        var result = Compile(source, target);
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

}
