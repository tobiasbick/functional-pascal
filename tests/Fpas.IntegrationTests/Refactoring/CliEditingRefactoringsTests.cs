// [Documentation](../../../docs/pascal/tools/refactoring.md)
using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Refactoring;
using System.Globalization;
using System.Text.Json;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class CliEditingRefactoringsTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Project = "Game/Game.fpasproj";
    private const string SourceFile = "Game/Bridge.fpas";

    [Fact]
    public async Task Extraction_replays_the_UTF16_range_into_a_typed_constant()
    {
        var source = """
            unit Demo.Bridge;
            uses System.Collections.Generic as Col;
            public function Make(): Col.List<int>;
            begin
              const Caption: string := '😀'; return new Col.List<int>(
              );
            end function;
            end unit;
            """.ReplaceLineEndings("\r\n");
        using var directory = RenameFixture.Create(sdk, source);
        var start = RenameFixture.Select(directory, source, "new Col.");
        var end = RenameFixture.Select(directory, source, ";\r\nend function;");
        var path = directory.File(".temp-data/extract.json");
        var preview = await RenameFixture.Cli(directory, "extract-variable", Project, "--file", SourceFile,
            "--line", Coordinate(start.Line), "--column", Coordinate(start.Column),
            "--end-line", Coordinate(end.Line), "--end-column", Coordinate(end.Column),
            "--name", "Items", "--plan", path);
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        Assert.Contains("Extract variable Items: planned", preview.Output);
        Assert.Equal(source, File.ReadAllText(start.File));

        var plan = RefactorPlanJson.Read(File.ReadAllBytes(path));
        Assert.Equal("extract-variable", plan.Kind);
        Assert.Equal("Items", plan.NewName);
        Assert.Equal(start.Column, plan.Selection!.Column);
        Assert.Equal(new RefactorPosition(end.Line, end.Column), plan.SelectionEnd);
        Assert.Equal(2, plan.EditCount);

        var applied = await RenameFixture.Cli(directory, "apply", Project, "--plan", path, "--no-restore", "--report", "json");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        using (var report = JsonDocument.Parse(applied.Output))
            Assert.True(report.RootElement.GetProperty("rechecked").GetBoolean());
        var expected = source.Replace("return new Col.List<int>(\r\n  );",
            "const Items: Col.List<int> := new Col.List<int>(\r\n  ); return Items;");
        Assert.Equal(expected, File.ReadAllText(start.File));

        var stale = await RenameFixture.Cli(directory, "apply", Project, "--plan", path, "--no-restore", "--report", "json");
        Assert.Equal(1, stale.ExitCode);
        using var rejected = JsonDocument.Parse(stale.Output);
        Assert.Equal("stale_plan", rejected.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal(expected, File.ReadAllText(start.File));
    }

    [Fact]
    public async Task Import_organization_is_file_scoped_and_reports_comment_retention()
    {
        var source = """
            unit Demo.Bridge;
            uses System.Text as Drop;
            uses System.Collections.Generic as Col;
            // Keep the attached note.
            uses System.IO as Retain;
            public function Make(): Col.List<int>;
            begin return new Col.List<int>(); end function;
            end unit;
            """.ReplaceLineEndings("\r\n");
        using var directory = RenameFixture.Create(sdk, source);
        var file = directory.File(SourceFile);
        var path = directory.File(".temp-data/imports.json");
        var preview = await RenameFixture.Cli(directory, "organize-imports", Project, "--file", SourceFile, "--plan", path, "--report", "json");
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        Assert.Equal(source, File.ReadAllText(file));
        var plan = RefactorPlanJson.Read(File.ReadAllBytes(path));
        Assert.Equal("organize-imports", plan.Kind);
        Assert.Null(plan.SelectionEnd);
        Assert.Contains(plan.Notes!, note => note.Contains("adjacent comment", StringComparison.Ordinal));
        var edit = Assert.Single(Assert.Single(plan.Files).Edits);
        Assert.Equal("uses System.Text as Drop;\r\n", edit.ExpectedText);
        Assert.Empty(edit.NewText);

        var applied = await RenameFixture.Cli(directory, "apply", Project, "--plan", path, "--no-restore");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        Assert.Contains("Organize imports: applied", applied.Output);
        Assert.Equal(source.Replace(edit.ExpectedText, ""), File.ReadAllText(file));

        var nextPath = directory.File(".temp-data/imports-clean.json");
        var unchanged = await RenameFixture.Cli(directory, "organize-imports", Project, "--file", SourceFile,
            "--plan", nextPath, "--no-restore");
        Assert.True(unchanged.ExitCode == 0, unchanged.Output + unchanged.Error);
        Assert.Empty(RefactorPlanJson.Read(File.ReadAllBytes(nextPath)).Files);
    }

    private static string Coordinate(int value) => value.ToString(CultureInfo.InvariantCulture);
}
