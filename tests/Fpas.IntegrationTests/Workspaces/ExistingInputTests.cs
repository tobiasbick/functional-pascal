using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class ExistingInputTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Existing_inputs_skip_restore_dependency_builds_and_shared_source_generation_including_reverification()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Game.fpasproj");
        var project = XDocument.Load(path);
        project.Root!.Add(new XElement("ItemGroup", new XElement("Compile", new XAttribute("Include", "$(IntermediateOutputPath)Generated.fpas"))),
            new XElement("Target", new XAttribute("Name", "GenerateSource"), new XAttribute("BeforeTargets", "FpasPrepareCompilation"),
                new XElement("WriteLinesToFile", new XAttribute("File", "$(IntermediateOutputPath)Generated.fpas"), new XAttribute("Lines", "unit Generated%3B end unit%3B"), new XAttribute("Overwrite", "true")),
                new XElement("Touch", new XAttribute("Files", "generated.marker"), new XAttribute("AlwaysCreate", "true"))));
        project.Save(path);
        var dependencyPath = directory.File("Shared/Shared.csproj"); var dependency = XDocument.Load(dependencyPath);
        dependency.Root!.Add(new XElement("Target", new XAttribute("Name", "ObserveBuild"), new XAttribute("BeforeTargets", "Build"),
            new XElement("Touch", new XAttribute("Files", "built.marker"), new XAttribute("AlwaysCreate", "true"))));
        dependency.Save(dependencyPath);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var prepared = await WorkspaceLoader.LoadAsync(new(path), deadline.Token);
        Assert.Empty(prepared.Issues); Assert.True(prepared.DependencyBuildsAllowed); Assert.True(prepared.RestorePerformed);
        Assert.True(File.Exists(directory.File("Game/generated.marker"))); Assert.True(File.Exists(directory.File("Shared/built.marker")));
        File.Delete(directory.File("Game/generated.marker")); File.Delete(directory.File("Shared/built.marker"));
        var outputs = Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories)
            .Where(file => file.Contains(Path.DirectorySeparatorChar + "bin" + Path.DirectorySeparatorChar, StringComparison.Ordinal) ||
                file.EndsWith("project.assets.json", StringComparison.Ordinal) || file.EndsWith("Generated.fpas", StringComparison.Ordinal))
            .ToDictionary(file => file, file => (Bytes: File.ReadAllBytes(file), Stamp: File.GetLastWriteTimeUtc(file)));
        var evaluated = await WorkspaceLoader.LoadAsync(new(path, Preparation: ContextPreparation.ExistingInputs), deadline.Token);
        Assert.Empty(evaluated.Issues); Assert.False(evaluated.DependencyBuildsAllowed); Assert.False(evaluated.RestorePerformed);
        Assert.True(WorkspaceAnalysis.Analyze(evaluated, deadline.Token).Success);
        Assert.Contains(Assert.Single(evaluated.Projects).Request.Sources, source => source.Path.EndsWith("Generated.fpas", StringComparison.Ordinal));
        await evaluated.VerifyInputsAsync(deadline.Token);
        Assert.False(File.Exists(directory.File("Game/generated.marker"))); Assert.False(File.Exists(directory.File("Shared/built.marker")));
        Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
        foreach (var (file, original) in outputs)
        { Assert.Equal(original.Bytes, File.ReadAllBytes(file)); Assert.Equal(original.Stamp, File.GetLastWriteTimeUtc(file)); }
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task A_missing_package_SDK_is_reported_before_MSBuild_can_download_it(bool globalVersion)
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Game.fpasproj");
        var missing = "99.0.0-missing.g" + Guid.NewGuid().ToString("N");
        File.WriteAllText(path, File.ReadAllText(path).Replace("Fpas.NET.Sdk/" + sdk.Version, "Fpas.NET.Sdk" + (globalVersion ? "" : "/" + missing), StringComparison.Ordinal));
        if (globalVersion) File.WriteAllText(directory.File("global.json"), "{\"msbuild-sdks\":{\"Fpas.NET.Sdk\":\"" + missing + "\"}}");
        var original = Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories).Order().ToArray();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        var evaluated = await WorkspaceLoader.LoadAsync(new(path, Preparation: ContextPreparation.ExistingInputs), deadline.Token);
        Assert.Empty(evaluated.Projects); Assert.False(evaluated.RestorePerformed);
        Assert.Equal("preparation_required", Assert.Single(evaluated.Issues).Code);
        Assert.Equal(original, Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories).Order().ToArray());
    }

    [Fact]
    public async Task Missing_dependency_metadata_requires_explicit_preparation_and_is_not_rebuilt()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Game.fpasproj");
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var prepared = await WorkspaceLoader.LoadAsync(new(path), deadline.Token); Assert.Empty(prepared.Issues);
        var metadata = Assert.Single(prepared.DependencyOutputs).Path; Assert.True(File.Exists(metadata)); File.Delete(metadata);
        var evaluated = await WorkspaceLoader.LoadAsync(new(path, Preparation: ContextPreparation.ExistingInputs), deadline.Token);
        Assert.Empty(evaluated.Projects); Assert.NotEmpty(evaluated.Issues);
        Assert.False(File.Exists(metadata)); Assert.False(evaluated.RestorePerformed);
    }

    [Fact]
    public async Task Existing_FPAS_metadata_must_match_current_dependency_sources_without_rebuilding_them()
    {
        using var directory = sdk.CreateProject();
        const string source = "unit Demo.Bridge; public function Read(): int; begin return 42; end function; end unit;";
        File.WriteAllText(directory.File("Game/Bridge.fpas"), source);
        Directory.CreateDirectory(directory.File("Consumer"));
        File.WriteAllText(directory.File("Consumer/Main.fpas"), "unit Consumer; uses Demo.Bridge as B; public function Read(): int; begin return B.Read(); end function; end unit;");
        var path = directory.File("Consumer/Consumer.fpasproj");
        File.WriteAllText(path, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup><ItemGroup><ProjectReference Include=\"../Game/Game.fpasproj\" /></ItemGroup></Project>");
        var build = await sdk.Run(directory, "build", path, "--disable-build-servers"); Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var metadata = directory.File("Game/bin/Debug/net10.0/Game.dll"); var bytes = File.ReadAllBytes(metadata); var stamp = File.GetLastWriteTimeUtc(metadata);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var input = new ContextOptions(path, Preparation: ContextPreparation.ExistingInputs);
        var evaluated = await WorkspaceLoader.LoadAsync(input, deadline.Token);
        Assert.Empty(evaluated.Issues); Assert.Equal(2, evaluated.Projects.Count);
        Assert.True(WorkspaceAnalysis.Analyze(evaluated, deadline.Token).Success);
        await evaluated.VerifyInputsAsync(deadline.Token);
        File.WriteAllText(directory.File("Game/Bridge.fpas"), source.Replace("42", "43", StringComparison.Ordinal));
        Assert.Equal("inputs_changed", (await Assert.ThrowsAsync<ContextFailure>(() => WorkspaceLoader.LoadAsync(input, deadline.Token))).Code);
        Assert.Equal(bytes, File.ReadAllBytes(metadata)); Assert.Equal(stamp, File.GetLastWriteTimeUtc(metadata));
    }
}
