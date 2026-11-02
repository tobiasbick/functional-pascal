using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task SDK_copies_tracks_repairs_disables_and_cleans_portable_symbols()
    {
        using var project = sdk.CreateProject();
        async Task Build(params string[] options)
        {
            var result = await sdk.Run(project, ["build", "Game.slnx", .. options]);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
        }
        await Build();
        var intermediate = project.File("Game/obj/Debug/net10.0/Game.pdb");
        var output = project.File("Game/bin/Debug/net10.0/Game.pdb");
        var consumer = project.File("Host/bin/Debug/net10.0/Game.pdb");
        Assert.True(File.Exists(intermediate)); Assert.Equal(File.ReadAllBytes(intermediate), File.ReadAllBytes(output));
        Assert.Equal(File.ReadAllBytes(intermediate), File.ReadAllBytes(consumer));
        var original = File.ReadAllBytes(intermediate);
        File.Delete(intermediate); await Build("--no-restore"); Assert.Equal(original, File.ReadAllBytes(intermediate));
        File.WriteAllText(intermediate, "corrupt symbols"); await Build("--no-restore"); Assert.Equal(original, File.ReadAllBytes(intermediate));
        await Build("--no-restore", "-p:DebugType=none"); Assert.False(File.Exists(intermediate)); Assert.False(File.Exists(output));
        await Build("--no-restore", "-p:DebugType=portable"); Assert.Equal(original, File.ReadAllBytes(intermediate));
        var clean = await sdk.Run(project, "clean", "Game.slnx"); Assert.True(clean.ExitCode == 0, clean.Output + clean.Error);
        Assert.False(File.Exists(intermediate)); Assert.False(File.Exists(output)); Assert.False(File.Exists(consumer));
    }
}
