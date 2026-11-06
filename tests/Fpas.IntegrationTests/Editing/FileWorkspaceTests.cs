using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Refactoring;
using Fpas.Tests.Support;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;
using Fpas.Workspaces.Snapshots;

namespace Fpas.IntegrationTests.Editing;

public sealed class FileWorkspaceTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Shared_correction_workflow_publishes_once_rechecks_and_preserves_root_outputs()
    {
        using var directory = sdk.CreateProject();
        var build = await sdk.Run(directory, "build", "Game/Game.fpasproj", "--disable-build-servers"); Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var outputs = Directory.GetFiles(directory.File("Game/bin"), "*", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(directory.File("Game/obj"), "fpas.compile.json*", SearchOption.AllDirectories)).ToDictionary(file => file, File.ReadAllBytes);
        var path = directory.File("Game/Bridge.fpas");
        const string source = "\uFEFFunit Demo.Bridge; public function Read(Value: integer): int; begin return Value; end function; end unit;\r\n";
        File.WriteAllText(path, source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var workspace = await FileWorkspace.OpenAsync(new(directory.File("Game/Game.fpasproj"), NoRestore: true), deadline.Token);
        var plan = workspace.Corrections(deadline.Token); Assert.Equal(source, Assert.Single(plan.Edits.Documents).Original.Text);
        Assert.Equal(System.Text.Encoding.UTF8.GetBytes(source), File.ReadAllBytes(path));
        await workspace.ApplyAsync(plan.Edits, token: deadline.Token);
        Assert.True(workspace.Analysis!.Success); Assert.True(workspace.Rechecked); Assert.Equal([path], workspace.Written);
        Assert.Empty(workspace.Corrections(deadline.Token).Edits.Documents);
        await Assert.ThrowsAsync<InvalidOperationException>(() => workspace.ApplyAsync(plan.Edits, token: deadline.Token));
        foreach (var (file, bytes) in outputs) Assert.Equal(bytes, File.ReadAllBytes(file));
    }

    [Fact]
    public async Task Typed_rename_validation_rejects_modified_foreign_and_stale_plans_without_plan_files()
    {
        const string source = "unit Demo.Bridge; public function Read(): int; begin return 42; end function; end unit;";
        using var directory = RenameFixture.Create(sdk, source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var workspace = await FileWorkspace.OpenAsync(new(directory.File("Game/Game.fpasproj")), deadline.Token);
        var plan = workspace.Rename(RenameFixture.Select(directory, source, "Read():"), "Fetch", deadline.Token).Plan;
        Assert.Equal(plan.Id, workspace.ValidateRename(plan, deadline.Token).Plan.Id);
        Assert.Equal("invalid_plan", Assert.Throws<ContextFailure>(() => workspace.ValidateRename(plan with { NewName = "Other" }, deadline.Token)).Code);
        var foreign = plan with { Configuration = "Release" }; foreign = foreign with { Id = RenamePlanJson.Identity(foreign) };
        Assert.Equal("plan_context_mismatch", Assert.Throws<ContextFailure>(() => workspace.ValidateRename(foreign, deadline.Token)).Code);
        File.AppendAllText(directory.File("Game/Bridge.fpas"), "\n// changed\n");
        var fresh = await FileWorkspace.OpenAsync(new(directory.File("Game/Game.fpasproj"), NoRestore: true), deadline.Token);
        Assert.Equal("stale_plan", Assert.Throws<ContextFailure>(() => fresh.ValidateRename(plan, deadline.Token)).Code);
        Assert.Empty(Directory.GetFiles(directory.Path, "*.json"));
    }

    [Fact]
    public async Task Publication_rejects_edits_from_a_different_text_snapshot()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Bridge.fpas");
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var workspace = await FileWorkspace.OpenAsync(new(directory.File("Game/Game.fpasproj")), deadline.Token);
        var foreign = new WorkspaceTextSnapshot([new(path, "foreign")]);
        var edits = TextEditSet.Create(foreign, [new(new(path, 1, 1, 7, Offset: 0), "foreign", "text")]);
        Assert.Equal("inputs_changed", (await Assert.ThrowsAsync<ContextFailure>(() => workspace.ApplyAsync(edits, token: deadline.Token))).Code);
        Assert.Empty(workspace.Written); Assert.False(workspace.Rechecked); Assert.NotNull(workspace.Analysis);
    }
}
