using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests;

public sealed class DocumentationTests
{
    [Fact]
    public void Global_documentation_uses_field_ids_and_omits_private_fields_and_initializers()
    {
        var result = Compile("""
            unit State;
            // Maximum value.
            public const Limit: int := 100;
            // Mutable count.
            public var Count: long := 0L;
            // Private state.
            var Hidden: int := 1;
            end unit;
            """);
        Assert.True(result.Success, string.Join(';', result.Diagnostics)); Assert.Empty(result.Diagnostics);
        var members = XDocument.Parse(System.Text.Encoding.UTF8.GetString(result.Program!.XmlDocumentation!)).Descendants("member").ToArray();
        Assert.Equal(new[] { "F:State.Count", "F:State.Limit" }, members.Select(member => (string)member.Attribute("name")!));
        Assert.Contains("Mutable count.", members[0].Value); Assert.Contains("Maximum value.", members[1].Value);
    }
    [Fact]
    public void Scalar_docs_use_real_type_and_enum_field_ids_and_aliases_use_the_target_signature()
    {
        var result = Compile("""
            unit Domains;
            // Transparent number.
            public type Count = ulong;
            // Account identity.
            public type Id = distinct long;
            // Network port.
            public type Port = ushort range 1..65535;
            // Direction.
            public type Direction = enum : byte
              // First direction.
              North;
            end enum;
            // Read the number.
            public function Read(Value: Count; IdValue: Id; Cost: decimal; Ratio: float): Count;
            begin return Value; end function;
            end unit;
            """);
        Assert.True(result.Success, string.Join(';', result.Diagnostics)); Assert.Empty(result.Diagnostics);
        var members = XDocument.Parse(System.Text.Encoding.UTF8.GetString(result.Program!.XmlDocumentation!)).Descendants("member");
        Assert.Equal(new[] { "F:Domains.Direction.North", "M:Domains.Read(System.UInt64,Domains.Id,System.Decimal,System.Single)",
            "T:Domains.Direction", "T:Domains.Id", "T:Domains.Port" }, members.Select(member => (string)member.Attribute("name")!));
    }
    [Fact]
    public void Public_XML_uses_CLR_ids_for_overloads_generics_arrays_byrefs_properties_and_union_factories()
    {
        const string source = """
            // Public API container.
            unit Demo.Api;
            // A generic box.
            // ## Type parameters
            // - `T`: Stored value.
            public type Box<T> = record
              // Stored property.
              Value: T;
              // Private state.
              private Secret: int := 0;
              // Copies a value.
              // ## Parameters
              // - `Item`: Current value.
              // - `Previous`: Previous value.
              // - `Other`: Returned value.
              // ## Type parameters
              // - `U`: Method type.
              // ## Returns
              // The other value.
              function Copy<U>(ref Item: T; out Previous: T; in Other: U): U;
              begin Previous := Item; return Other; end function;
            end record;
            // Optional result.
            public type Outcome<T> = union
              // Empty result.
              None;
              // Present result.
              // ## Parameters
              // - `Value`: Payload.
              Some(Value: T);
            end union;
            // Integer overload.
            public function Read(Value: int): int; begin return Value; end function;
            // Generic overload.
            public function Read<T>(Value: T): T; begin return Value; end function;
            // Array and nullable arguments.
            public function Arrays(Grid: int[,]; Text: string?[][]; Number: double?): int; begin return 0; end function;
            // Private helper.
            private function Hidden(): int; begin return 1; end function;
            end unit;
            """;
        var result = Compile(source);
        Assert.True(result.Success, string.Join(';', result.Diagnostics)); Assert.Empty(result.Diagnostics);
        var document = XDocument.Parse(System.Text.Encoding.UTF8.GetString(result.Program!.XmlDocumentation!));
        Assert.Equal("Docs", document.Root!.Element("assembly")!.Element("name")!.Value);
        var members = document.Root.Element("members")!.Elements("member").ToDictionary(member => (string)member.Attribute("name")!);
        Assert.Equal(new[]
        {
            "M:Demo.Api.Arrays(System.Int32[0:,0:],System.String[][],System.Nullable{System.Double})",
            "M:Demo.Api.Box`1.Copy``1(`0@,`0@,``0@)", "M:Demo.Api.Outcome`1.Some(`0)",
            "M:Demo.Api.Read(System.Int32)", "M:Demo.Api.Read``1(``0)", "P:Demo.Api.Box`1.Value",
            "P:Demo.Api.Outcome`1.None", "T:Demo.Api", "T:Demo.Api.Box`1", "T:Demo.Api.Outcome`1"
        }, members.Keys.ToArray());
        var copy = members["M:Demo.Api.Box`1.Copy``1(`0@,`0@,``0@)"];
        Assert.Equal(new[] { "Item", "Previous", "Other" }, copy.Elements("param").Select(parameter => (string)parameter.Attribute("name")!));
        Assert.Equal("U", (string)copy.Element("typeparam")!.Attribute("name")!);
        Assert.Equal("The other value.", copy.Element("returns")!.Value);
        var plain = Compile(source, enabled: false).Program!;
        Assert.Null(plain.XmlDocumentation); Assert.Equal(plain.AssemblyImage, result.Program.AssemblyImage);
        Assert.Equal(result.Program.XmlDocumentation, Compile(source).Program!.XmlDocumentation);
    }

    [Theory]
    [InlineData("\n")]
    [InlineData("\r\n")]
    [InlineData("\r")]
    public void Only_adjacent_standalone_blocks_attach_before_modifiers_and_comment_text_stays_Markdown(string newline)
    {
        var source = string.Join(newline,
            "// Header.", "unit Attached;", "uses System.Threading.Tasks as Tasks;",
            "// Detached.", "", "public function Detached(): int; begin return 0; end function;",
            "// Attached summary.", "//", "// Second paragraph with `code` & <literal>.", "public async function Read(@type: int): Tasks.Task<int>;",
            "begin return @type; end function; // Inline comment.",
            "public", "// Within modifiers, ignored.", "function Plain(): string; begin return 'https://example.com/'; end function;",
            "end unit;", "// EOF comment");
        var result = Compile(source);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        var members = XDocument.Parse(System.Text.Encoding.UTF8.GetString(result.Program!.XmlDocumentation!)).Descendants("member").ToArray();
        Assert.Equal(new[] { "M:Attached.Read(System.Int32)", "T:Attached" }, members.Select(member => (string)member.Attribute("name")!));
        Assert.Equal(2, members[0].Element("summary")!.Elements("para").Count());
        Assert.Equal("code", members[0].Descendants("c").Single().Value);
        Assert.Contains("& <literal>", members[0].Value);
    }

    [Fact]
    public void Markdown_sections_escape_XML_render_lists_links_and_fences_and_use_escaped_source_names()
    {
        var result = Compile("""
            unit Markdown;
            // First paragraph & <tag>.
            //
            // - First `item`.
            // - [Reference](https://example.com/a(b)?x=1&y=2)
            //
            // ```pascal
            // ## Returns
            // <raw>& code
            // ```
            // ## Parameters
            // - `@type`: First paragraph.
            //
            //   Another paragraph with `value`.
            // ## Returns
            // The task and its result.
            // ## Remarks
            // 1. One.
            // 2. Two.
            public function Read(@type: int): int; begin return @type; end function;
            end unit;
            """);
        Assert.True(result.Success, string.Join(';', result.Diagnostics)); Assert.Empty(result.Diagnostics);
        var member = XDocument.Parse(System.Text.Encoding.UTF8.GetString(result.Program!.XmlDocumentation!)).Descendants("member").Single();
        Assert.Equal("type", (string)member.Element("param")!.Attribute("name")!);
        Assert.Equal(2, member.Element("param")!.Elements("para").Count());
        Assert.Equal("https://example.com/a(b)?x=1&y=2", (string)member.Descendants("see").Single().Attribute("href")!);
        Assert.Equal("## Returns\n<raw>& code", member.Descendants("code").Single().Value);
        Assert.Equal(new[] { "bullet", "number" }, member.Descendants("list").Select(list => (string)list.Attribute("type")!));
        Assert.Empty(member.Descendants("raw")); Assert.Empty(member.Descendants("tag"));
    }

    [Fact]
    public void Invalid_sections_warn_with_source_locations_keep_first_valid_entries_and_preserve_other_text()
    {
        var result = Compile("""
            unit Warnings;
            // Summary.
            // ## Parameters
            // - `Value`: Valid first.
            // - `Value`: Duplicate.
            // - `value`: Wrong case.
            // malformed item
            // ## Returns
            // Valid return.
            // ## Returns
            // Duplicate return.
            public function Read(Value: int): int; begin return Value; end function;
            // ## Returns
            // Not a function.
            private procedure Hidden();
            begin
              // ## Returns
              // Not a function either.
              const Local: int := 1;
            end procedure;
            end unit;
            """);
        Assert.True(result.Success); Assert.Equal(6, result.Diagnostics.Count);
        Assert.All(result.Diagnostics, diagnostic => { Assert.Equal("FP6001", diagnostic.Code); Assert.Equal(DiagnosticSeverity.Warning, diagnostic.Severity); Assert.Equal("docs.fpas", diagnostic.Location.Path); });
        Assert.Equal(new[] { 5, 6, 7, 10, 13, 17 }, result.Diagnostics.Select(diagnostic => diagnostic.Location.Line).Order());
        var member = XDocument.Parse(System.Text.Encoding.UTF8.GetString(result.Program!.XmlDocumentation!)).Descendants("member").Single();
        Assert.Equal("Valid first.", member.Element("param")!.Value); Assert.Equal("Valid return.", member.Element("returns")!.Value);
        Assert.Contains("Duplicate.", member.Element("remarks")!.Value); Assert.Contains("Wrong case.", member.Element("remarks")!.Value);
    }

    [Theory]
    [InlineData(false, "", "", false, "warning")]
    [InlineData(true, "", "", false, "error")]
    [InlineData(false, "FP6001", "", false, "error")]
    [InlineData(true, "", "6001", false, "warning")]
    [InlineData(true, "", "", true, "none")]
    public void Documentation_warning_policy_controls_suppression_promotion_and_output(bool promote, string errors, string exceptions, bool suppress, string expected)
    {
        const string source = "unit Policy; // inline\n// ## Returns\n// Invalid.\npublic procedure P(); begin end procedure; end unit;";
        var result = Compile(source, options: new(promote, errors.Length == 0 ? [] : [errors], exceptions.Length == 0 ? [] : [exceptions], suppress ? ["FP6001"] : []));
        Assert.Equal(expected != "error", result.Success);
        if (expected == "none") Assert.Empty(result.Diagnostics);
        else Assert.Equal(expected == "error" ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning, Assert.Single(result.Diagnostics).Severity);
        Assert.Equal(expected == "error", result.Program is null);
        var disabled = Compile(source, enabled: false, options: new(true)); Assert.True(disabled.Success); Assert.Empty(disabled.Diagnostics);
    }

    private static CompilationResult Compile(string text, bool enabled = true, DiagnosticOptions? options = null)
        => FpasCompiler.Compile(new([new("docs.fpas", text)], "Docs", [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location],
            CompilationTarget.Library, EmitDocumentation: enabled, DiagnosticOptions: options));
}
