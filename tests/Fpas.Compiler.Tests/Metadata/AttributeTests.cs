using Fpas.Compiler.Tests.Support;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Metadata;

public sealed class AttributeTests
{
    [Theory]
    [InlineData("[Sys.FlagsAttribute] type Flag = enum A = 1; B = 2; end enum;")]
    [InlineData("[Sys.SerializableAttribute] type R = record end record;")]
    [InlineData("type R = record [Analysis.MaybeNullAttribute] Value: string?; [Analysis.NotNullAttribute] Other: string; end record;")]
    [InlineData("[Sys.ObsoleteAttribute('old', false)] type R = record [Json.JsonPropertyNameAttribute('value')] Value: int; end record;")]
    [InlineData("[Sys.ObsoleteAttribute('one' + #10 + 'two')] procedure P(); begin end procedure;")]
    [InlineData("[Sys.ObsoleteAttribute('old', DiagnosticId := 'TEST001', UrlFormat := 'https://example.com/{0}')] procedure P(); begin end procedure;")]
    [InlineData("type R<T> = record [Sys.ObsoleteAttribute('old')] static function Get(): int; begin return 1; end function; end record;")]
    [InlineData("[Sys.ObsoleteAttribute('old')] type U<T> = union Empty; Value(Data: T); end union;")]
    [InlineData("[Sys.ObsoleteAttribute('old')] type Id = distinct int;")]
    [InlineData("[Sys.ObsoleteAttribute('old')] type Name = distinct string;")]
    [InlineData("[Sys.ObsoleteAttribute('old')] type Percent = int range 0..100;")]
    [InlineData("procedure Outer(); begin [Sys.ObsoleteAttribute('old')] procedure Inner(); begin end procedure; Inner(); end procedure;")]
    [InlineData("procedure Outer(); begin [procedure() begin end procedure][0](); end procedure;")]
    public void Existing_attribute_syntax_supports_emitted_types_routines_and_record_properties(string declarations)
    {
        var result = Compile(declarations);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
    }

    [Theory]
    [InlineData("[Sys.ObsoleteAttribute('a')] [Sys.ObsoleteAttribute('b')] procedure P(); begin end procedure;")]
    [InlineData("[Sys.FlagsAttribute] type R = record end record;")]
    [InlineData("[Sys.FlagsAttribute] procedure P(); begin end procedure;")]
    [InlineData("[Sys.ObsoleteAttribute] const Value: int := 1;")]
    [InlineData("[Sys.ObsoleteAttribute] type Alias = int;")]
    [InlineData("[Sys.Attribute] procedure P(); begin end procedure;")]
    [InlineData("[Sys.String] procedure P(); begin end procedure;")]
    [InlineData("[Sys.ObsoleteAttribute('old', Missing := 1)] procedure P(); begin end procedure;")]
    [InlineData("[Sys.ObsoleteAttribute('old', DiagnosticId := 'a', DiagnosticId := 'b')] procedure P(); begin end procedure;")]
    [InlineData("[Sys.ObsoleteAttribute(DiagnosticId := 'a', 'old')] procedure P(); begin end procedure;")]
    [InlineData("[Sys.ObsoleteAttribute(Sys.String.Empty)] procedure P(); begin end procedure;")]
    [InlineData("[Sys.ObsoleteAttribute(int.Parse('1').ToString())] procedure P(); begin end procedure;")]
    [InlineData("type R = record [Sys.NonSerializedAttribute] Value: int; end record;")]
    [InlineData("type R = record [Analysis.MaybeNullAttribute] Value: string; end record;")]
    [InlineData("type R = record [Analysis.NotNullAttribute] Value: string?; end record;")]
    public void Invalid_attribute_targets_duplicates_members_and_runtime_values_are_rejected(string declarations)
    {
        var result = Compile(declarations);
        Assert.False(result.Success); Assert.Equal("FP3035", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Attributes_preserve_documentation_before_the_first_attribute_and_emit_deterministically()
    {
        const string text = """
            unit Marked;
            uses System as Sys, System.Text.Json.Serialization as Json;
            // A documented record.
            [Sys.ObsoleteAttribute('record')]
            public type R = record
              // A documented property.
              [Json.JsonPropertyNameAttribute('payload')]
              Value: int;
            end record;
            // A documented routine.
            [Sys.ObsoleteAttribute('routine')]
            public function Get(): int; begin return 1; end function;
            [Sys.ObsoleteAttribute('undocumented')]
            // This comment is within the modifier sequence.
            public procedure Other(); begin end procedure;
            end unit;
            """;
        CompilationResult Run(IEnumerable<string> references, bool documentation = true) => FpasCompiler.Compile(new([new("marked.fpas", text)], "Marked",
            references.ToArray(), CompilationTarget.Library, EmitDebugInformation: true, EmitDocumentation: documentation));
        string[] references = [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location];
        var first = Run(references); var second = Run(references.Reverse()); var plain = Run(references, false);
        Assert.True(first.Success, string.Join(';', first.Diagnostics)); Assert.True(second.Success); Assert.True(plain.Success);
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage); Assert.Equal(first.Program.PortablePdb, second.Program.PortablePdb);
        Assert.Equal(first.Program.AssemblyImage, plain.Program!.AssemblyImage);
        var members = XDocument.Parse(Encoding.UTF8.GetString(first.Program.XmlDocumentation!)).Descendants("member").ToArray();
        Assert.Equal(3, members.Length); Assert.Contains(members, member => (string?)member.Attribute("name") == "P:Marked.R.Value");
        using var pe = new PEReader(new MemoryStream(first.Program.AssemblyImage)); var metadata = pe.GetMetadataReader();
        var property = metadata.PropertyDefinitions.Select(metadata.GetPropertyDefinition).Single(property => metadata.GetString(property.Name) == "Value");
        Assert.Equal(2, property.GetCustomAttributes().Count);
        Assert.DoesNotContain(metadata.FieldDefinitions.Select(metadata.GetFieldDefinition), field => field.GetCustomAttributes().Count != 0);
    }

    private static CompilationResult Compile(string declarations) => TestCompilation.Compile($$"""
        program Attributes;
        uses System as Sys, System.Text.Json.Serialization as Json, System.Diagnostics.CodeAnalysis as Analysis;
        {{declarations}}
        begin end;
        end program;
        """);
}
