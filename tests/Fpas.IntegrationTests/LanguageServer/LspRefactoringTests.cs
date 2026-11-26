// [Documentation](../../../docs/pascal/tools/language-server.md)
using Fpas.IntegrationTests.Build;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspRefactoringTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Uses the shared protocol implementation workflow against an unsaved document.</summary>
    [Fact]
    public async Task Protocol_implementation_previews_an_unsaved_record_and_returns_versioned_edits()
    {
        using var directory = sdk.CreateProject();
        var project = directory.File("Game/Game.fpasproj");
        File.WriteAllText(project, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        var path = directory.File("Game/Bridge.fpas");
        var original = "unit Adapter; end unit;";
        File.WriteAllText(path, original);
        const string unsaved = """
            unit Adapter;
            type IRead = protocol
              function Read(): int;
            end protocol;
            type Reader = record
            end record;
            end unit;
            """;
        var restored = await sdk.Run(directory, "restore", "Game/Game.fpasproj");
        Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
        await using var client = new LspProbe();
        await client.Initialize(project);
        await client.Notify("initialized", new { });
        await client.Open(path, unsaved, version: 5);
        var at = Position(unsaved, "Reader =");
        var preview = Result(await client.Request("fpas/prepareRefactoring", new
        {
            schemaVersion = 1, kind = "implement-protocol", textDocument = new { uri = new Uri(path).AbsoluteUri, version = 5 },
            position = at, end = at, protocol = "IRead", members = new[] { new { key = "Read", value = "return 42;" } }
        }));
        Assert.True(preview.GetProperty("canApply").GetBoolean());
        var applied = Result(await client.Request("fpas/applyRefactoring", new
        {
            schemaVersion = 1, id = preview.GetProperty("id").GetString()
        }));
        var change = Assert.Single(applied.GetProperty("edit").GetProperty("documentChanges").EnumerateArray());
        Assert.Equal(5, change.GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.Contains(change.GetProperty("edits").EnumerateArray(),
            edit => edit.GetProperty("newText").GetString() == " implements IRead");
        Assert.Contains(change.GetProperty("edits").EnumerateArray(),
            edit => edit.GetProperty("newText").GetString()!.Contains("return 42;", StringComparison.Ordinal));
        Assert.Equal(original, File.ReadAllText(path));
        await client.Stop();
    }

    [Fact]
    public async Task Extraction_spells_a_closed_CLR_type_and_import_cleanup_preserves_comments_and_CRLF()
    {
        using var directory = sdk.CreateProject();
        var project = directory.File("Types/Types.fpasproj");
        Directory.CreateDirectory(Path.GetDirectoryName(project)!);
        File.WriteAllText(project, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        var path = directory.File("Types/Types.fpas");
        var original = """
            unit Types;
            uses System.Text as Drop;
            uses System.Console as Log;
            uses System.Collections.Generic as Col;
            // Keep the attached note.
            uses System.IO as Retain;
            public function Make(): Col.List of int;
            begin
              Log.WriteLine('ready');
              return new Col.List of int();
            end function;
            end unit;
            """.ReplaceLineEndings("\r\n");
        File.WriteAllText(path, original);
        var restored = await sdk.Run(directory, "restore", "Types/Types.fpasproj");
        Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
        await using var client = new LspProbe();
        await client.Initialize(project);
        await client.Notify("initialized", new { });
        await client.Open(path, original);
        var textDocument = new { uri = new Uri(path).AbsoluteUri, version = 1 };
        var preview = Result(await client.Request("fpas/prepareRefactoring", new
        {
            schemaVersion = 1, kind = "extract-variable", textDocument,
            position = Position(original, "new Col."), end = Position(original, ";\r\nend function;"), name = "Items"
        }));
        var extracted = Assert.Single(preview.GetProperty("files").EnumerateArray()).GetProperty("edits");
        Assert.Equal("const Items: Col.List of int := new Col.List of int();\r\n  ",
            extracted[0].GetProperty("newText").GetString());
        var at = Position(original, "System.Text");
        var imports = Result(await client.Request("fpas/prepareRefactoring", new
            { schemaVersion = 1, kind = "organize-imports", textDocument, position = at, end = at }));
        var edit = Assert.Single(Assert.Single(imports.GetProperty("files").EnumerateArray()).GetProperty("edits").EnumerateArray());
        Assert.Equal("uses System.Text as Drop;\r\n", edit.GetProperty("expectedText").GetString());
        Assert.Equal("", edit.GetProperty("newText").GetString());
        Assert.Contains(imports.GetProperty("notes").EnumerateArray(), note => note.GetString()!.Contains("comment", StringComparison.Ordinal));
        Assert.Equal(original, File.ReadAllText(path));
        await client.Stop();
    }

    [Fact]
    public async Task Linked_owners_share_one_versioned_edit_set_and_closing_the_buffer_invalidates_it()
    {
        using var directory = sdk.CreateProject();
        foreach (var owner in new[] { "First", "Second" })
        {
            var project = directory.File(owner + "/" + owner + ".fpasproj");
            Directory.CreateDirectory(Path.GetDirectoryName(project)!);
            File.WriteAllText(project, $"""
                <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
                  <PropertyGroup>
                    <OutputType>Library</OutputType>
                    <EnableDefaultFpasCompileItems>false</EnableDefaultFpasCompileItems>
                  </PropertyGroup>
                  <ItemGroup><Compile Include="../Shared.fpas" /><Compile Include="Options.fpas" /></ItemGroup>
                </Project>
                """);
            var firstName = owner == "First" ? "X" : "Column";
            File.WriteAllText(directory.File(owner + "/Options.fpas"),
                $"unit Options; public function Map({firstName}: int; Y: int): int; begin return Y; end function; end unit;");
        }
        var context = directory.File("Linked.slnx");
        File.WriteAllText(context, """
            <Solution>
              <Configurations>
                <ProjectType Name="FPAS" Extension=".fpasproj" TypeId="{5D4B15AB-1024-4803-9015-4F069B13DE5A}" />
              </Configurations>
              <Project Path="First/First.fpasproj" />
              <Project Path="Second/Second.fpasproj" />
            </Solution>
            """);
        var path = directory.File("Shared.fpas");
        var original = """
            unit Shared;
            uses Options as Opt;
            function Ignore(Dead: int): int; begin return 5; end function;
            function Pair(First: int; Second: int): int; begin return First - Second; end function;
            function Sum(Value: int; Extra: int := 4): int;
            begin
              return Value + Extra;
            end function;
            public function Run(): int;
            begin
              const Offset: int := 7;
              const Text: string := '😀'; return Sum(2, 3) + Ignore(1) + Opt.Map(1, 2) + Pair(5, 2) + Offset;
            end function;
            end unit;
            """.ReplaceLineEndings("\r\n");
        File.WriteAllText(path, original);
        var restored = await sdk.Run(directory, "restore", "Linked.slnx");
        Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);

        await using var client = new LspProbe();
        await client.Initialize(context);
        await client.Notify("initialized", new { });
        await client.Open(path, original, version: 7);
        var position = Position(original, "Sum(Value");
        var planRequest = new { schemaVersion = 1, kind = "add-parameter",
            textDocument = new { uri = new Uri(path).AbsoluteUri, version = 7 }, position, end = position,
            projectUri = new Uri(directory.File("First/First.fpasproj")).AbsoluteUri, name = "Added", type = "int", argument = "99" };
        var preview = Result(await client.Request("fpas/prepareRefactoring", planRequest));
        var file = Assert.Single(preview.GetProperty("files").EnumerateArray());
        Assert.True(preview.GetProperty("canApply").GetBoolean());
        Assert.Equal(7, file.GetProperty("version").GetInt32());
        Assert.Equal(3, file.GetProperty("edits").GetArrayLength());
        var applyRequest = new { schemaVersion = 1, id = preview.GetProperty("id").GetString() };
        var result = Result(await client.Request("fpas/applyRefactoring", applyRequest));
        var document = Assert.Single(result.GetProperty("edit").GetProperty("documentChanges").EnumerateArray());
        Assert.Equal(7, document.GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.Equal(new[] { "Added: int; ", "Extra := ", ", Added := 99" },
            document.GetProperty("edits").EnumerateArray().Select(edit => edit.GetProperty("newText").GetString()));
        Error(await client.Request("fpas/applyRefactoring", applyRequest), -32801);

        var selectedCall = Position(original, "Sum(2, 3)");
        var naming = Result(await client.Request("fpas/prepareRefactoring", new {
            schemaVersion = 1, kind = "name-arguments", planRequest.textDocument, position = selectedCall, end = selectedCall, planRequest.projectUri
        }));
        var named = Result(await client.Request("fpas/applyRefactoring", new { schemaVersion = 1, id = naming.GetProperty("id").GetString() }));
        var namedFile = Assert.Single(named.GetProperty("edit").GetProperty("documentChanges").EnumerateArray());
        Assert.Equal(7, namedFile.GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.Equal(new[] { "Value := ", "Extra := " }, namedFile.GetProperty("edits").EnumerateArray()
            .Select(edit => edit.GetProperty("newText").GetString()));

        var divergent = Position(original, "Map(1, 2)");
        var rejected = await client.Request("fpas/prepareRefactoring", new {
            schemaVersion = 1, kind = "name-arguments", planRequest.textDocument, position = divergent, end = divergent, planRequest.projectUri
        });
        Error(rejected, -32803);
        Assert.Contains("Different owners", rejected.GetProperty("error").GetProperty("message").GetString());

        var removed = Position(original, "Dead: int");
        var signature = Result(await client.Request("fpas/prepareRefactoring", new {
            schemaVersion = 1, kind = "change-signature", planRequest.textDocument, position = removed, end = removed,
            planRequest.projectUri, type = "long",
            arguments = new[] { new { position = Position(original, "Ignore(1)"), expression = "99" } }
        }));
        var signatureEdit = Result(await client.Request("fpas/applyRefactoring", new {
            schemaVersion = 1, id = signature.GetProperty("id").GetString()
        }));
        var signatureFile = Assert.Single(signatureEdit.GetProperty("edit").GetProperty("documentChanges").EnumerateArray());
        Assert.Equal(new[] { "long", "99" }, signatureFile.GetProperty("edits").EnumerateArray()
            .Select(edit => edit.GetProperty("newText").GetString()));

        var pair = Position(original, "Pair(5, 2)");
        var permutation = Result(await client.Request("fpas/prepareRefactoring", new {
            schemaVersion = 1, kind = "change-signature", planRequest.textDocument, position = pair, end = pair,
            planRequest.projectUri, order = new[] { "Second", "First" }
        }));
        var reordered = Result(await client.Request("fpas/applyRefactoring", new {
            schemaVersion = 1, id = permutation.GetProperty("id").GetString()
        }));
        var reorderFile = Assert.Single(reordered.GetProperty("edit").GetProperty("documentChanges").EnumerateArray());
        Assert.Equal(7, reorderFile.GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.Equal(new[] { "Second: int", "First: int", "First := ", "Second := " },
            reorderFile.GetProperty("edits").EnumerateArray().Select(edit => edit.GetProperty("newText").GetString()));

        var local = Position(original, "Offset: int");
        var inline = Result(await client.Request("fpas/prepareRefactoring", new {
            schemaVersion = 1, kind = "inline-variable", planRequest.textDocument, position = local, end = local, planRequest.projectUri
        }));
        var inlined = Result(await client.Request("fpas/applyRefactoring", new {
            schemaVersion = 1, id = inline.GetProperty("id").GetString()
        }));
        var inlineFile = Assert.Single(inlined.GetProperty("edit").GetProperty("documentChanges").EnumerateArray());
        Assert.Equal(7, inlineFile.GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.Equal(new[] { "", "(7)" }, inlineFile.GetProperty("edits").EnumerateArray()
            .Select(edit => edit.GetProperty("newText").GetString()));

        var removal = Result(await client.Request("fpas/prepareRefactoring", new {
            schemaVersion = 1, kind = "remove-parameter", planRequest.textDocument, position = removed, end = removed, planRequest.projectUri
        }));
        var removalFile = Assert.Single(removal.GetProperty("files").EnumerateArray());
        Assert.Equal(new[] { "Dead: int", "1" },
            removalFile.GetProperty("edits").EnumerateArray().Select(edit => edit.GetProperty("expectedText").GetString()));
        var removalEdit = Result(await client.Request("fpas/applyRefactoring", new { schemaVersion = 1, id = removal.GetProperty("id").GetString() }));
        var changes = Assert.Single(removalEdit.GetProperty("edit").GetProperty("documentChanges").EnumerateArray());
        Assert.Equal(7, changes.GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.All(changes.GetProperty("edits").EnumerateArray(), edit => Assert.Equal("", edit.GetProperty("newText").GetString()));

        var statement = Position(original, "return Sum(2");
        var extraction = Result(await client.Request("fpas/prepareRefactoring", new {
            schemaVersion = 1, kind = "extract-routine", planRequest.textDocument, position = statement,
            end = Position(original, "\r\nend function;", last: true), planRequest.projectUri, name = "Calculate"
        }));
        var extracted = Result(await client.Request("fpas/applyRefactoring", new {
            schemaVersion = 1, id = extraction.GetProperty("id").GetString()
        }));
        var extractedFile = Assert.Single(extracted.GetProperty("edit").GetProperty("documentChanges").EnumerateArray());
        Assert.Equal(7, extractedFile.GetProperty("textDocument").GetProperty("version").GetInt32());
        var extractedEdits = extractedFile.GetProperty("edits").EnumerateArray().Select(edit => edit.GetProperty("newText").GetString()).ToArray();
        Assert.StartsWith("  function Calculate(): int;\r\n", extractedEdits[0]);
        Assert.Equal("return Calculate();", extractedEdits[1]);

        preview = Result(await client.Request("fpas/prepareRefactoring", planRequest));
        await client.Close(path);
        Error(await client.Request("fpas/applyRefactoring", new { schemaVersion = 1, id = preview.GetProperty("id").GetString() }), -32801);
        Assert.Equal(original, File.ReadAllText(path));
        await client.Stop();
    }
}
