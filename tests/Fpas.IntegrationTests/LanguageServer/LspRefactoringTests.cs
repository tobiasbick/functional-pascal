using Fpas.IntegrationTests.Build;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspRefactoringTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
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
            uses System.Text as Drop, System.Console as Log, System.Collections.Generic as Col,
              // Keep the attached note.
              System.IO as Retain;
            public function Make(): Col.List<int>;
            begin
              Log.WriteLine('ready');
              return new Col.List<int>();
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
        Assert.Equal("const Items: Col.List<int> := new Col.List<int>();\r\n  ",
            extracted[0].GetProperty("newText").GetString());
        var at = Position(original, "System.Text");
        var imports = Result(await client.Request("fpas/prepareRefactoring", new
            { schemaVersion = 1, kind = "organize-imports", textDocument, position = at, end = at }));
        var edit = Assert.Single(Assert.Single(imports.GetProperty("files").EnumerateArray()).GetProperty("edits").EnumerateArray());
        Assert.Equal("System.Text as Drop, ", edit.GetProperty("expectedText").GetString());
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
                  <ItemGroup><Compile Include="../Shared.fpas" /></ItemGroup>
                </Project>
                """);
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
            function Sum(Value: int; Extra: int := 4): int;
            begin
              return Value + Extra;
            end function;
            public function Run(): int;
            begin
              const Text: string := '😀'; return Sum(2, 3);
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

        preview = Result(await client.Request("fpas/prepareRefactoring", planRequest));
        await client.Close(path);
        Error(await client.Request("fpas/applyRefactoring", new { schemaVersion = 1, id = preview.GetProperty("id").GetString() }), -32801);
        Assert.Equal(original, File.ReadAllText(path));
        await client.Stop();
    }
}
