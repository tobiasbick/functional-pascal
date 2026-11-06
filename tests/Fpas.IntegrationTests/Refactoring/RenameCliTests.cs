using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Refactoring;
using System.Text.Json;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class RenameCliTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Source = "unit Demo.Bridge; public function Read(Value: int): int; begin return Value; end function; public function Run(): int; begin return Read(42); end function; end unit;";

    [Fact]
    public async Task Planning_is_read_only_apply_rechecks_and_stale_or_modified_plans_never_write()
    {
        using var directory = RenameFixture.Create(sdk, Source); var selection = RenameFixture.Select(directory, Source, "Read(Value");
        var arguments = new[] { "rename", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "1", "--column", selection.Column.ToString(System.Globalization.CultureInfo.InvariantCulture),
            "--new-name", "Fetch", "--plan", ".temp-data/rename.json", "--report", "json" };
        var planned = await RenameFixture.Cli(directory, arguments); Assert.True(planned.ExitCode == 0, planned.Output + planned.Error); Assert.Empty(planned.Error);
        var path = directory.File(".temp-data/rename.json"); var original = File.ReadAllBytes(path);
        var plan = RenamePlanJson.Read(original); Assert.Equal(2, plan.EditCount); Assert.True(plan.ChangesPublicApi);
        Assert.Equal(Source, File.ReadAllText(directory.File("Game/Bridge.fpas"))); Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
        var overwrite = await RenameFixture.Cli(directory, arguments); Assert.Equal(1, overwrite.ExitCode); Assert.Equal(original, File.ReadAllBytes(path));
        var mismatch = await RenameFixture.Cli(directory, "apply", "Missing.fpasproj", "--plan", path, "--report", "json");
        Assert.Equal(1, mismatch.ExitCode); Assert.False(File.Exists(directory.File("Missing.fpasproj")));
        using (var json = JsonDocument.Parse(mismatch.Output)) Assert.Equal("plan_context_mismatch", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        File.WriteAllText(path, System.Text.Encoding.UTF8.GetString(original).Replace("Fetch", "Broken", StringComparison.Ordinal));
        var modified = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--report", "json");
        Assert.Equal(1, modified.ExitCode); using (var json = JsonDocument.Parse(modified.Output)) Assert.Equal("invalid_plan", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        File.WriteAllBytes(path, original);
        var applied = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--no-restore", "--report", "json");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error); Assert.Empty(applied.Error);
        using (var json = JsonDocument.Parse(applied.Output))
        { Assert.True(json.RootElement.GetProperty("rechecked").GetBoolean()); Assert.All(json.RootElement.GetProperty("files").EnumerateArray(), file => Assert.True(file.GetProperty("written").GetBoolean())); }
        Assert.Equal(Source.Replace("Read", "Fetch", StringComparison.Ordinal), File.ReadAllText(directory.File("Game/Bridge.fpas")));
        var stale = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--no-restore", "--report", "json");
        Assert.Equal(1, stale.ExitCode); using (var json = JsonDocument.Parse(stale.Output)) Assert.Equal("stale_plan", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Theory]
    [InlineData("bad.name")]
    [InlineData("@end")]
    [InlineData("_")]
    [InlineData("1Bad")]
    public async Task Invalid_names_fail_before_preparation(string name)
    {
        using var directory = new Fpas.Tests.Support.TemporaryDirectory();
        var result = await RenameFixture.Cli(directory, "rename", "Missing.fpasproj", "--file", "Missing.fpas", "--line", "1", "--column", "1", "--new-name", name, "--plan", "rename.json", "--report", "json");
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error); Assert.Empty(Directory.GetFileSystemEntries(directory.Path));
        using var json = JsonDocument.Parse(result.Output); Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }
}
