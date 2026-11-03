using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Cli;

public sealed class CompilerOutputOwnershipTests
{
    [Theory]
    [InlineData(".dll", "exe", "portable")]
    [InlineData(".pdb", "exe", "portable")]
    [InlineData(".pdb", "exe", "none")]
    [InlineData(".runtimeconfig.json", "exe", "portable")]
    [InlineData(".runtimeconfig.json", "library", "portable")]
    public async Task Reference_assemblies_cannot_be_replaced_or_deleted_by_output_publication(string extension, string target, string debug)
    {
        using var directory = new TemporaryDirectory();
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var dependency = directory.File("Dependency.dll");
        var prepared = await ProcessRunner.RunAsync([cli, "compile", "-", "--output", dependency, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--diagnostics", "json"],
            "unit Dependency; public function Value(): int; begin return 1; end function; end unit;");
        Assert.True(prepared.ExitCode == 0, prepared.Output + prepared.Error);
        var original = File.ReadAllBytes(dependency);
        var output = directory.File("Candidate.dll");
        var reference = Path.ChangeExtension(output, extension);
        File.Copy(dependency, reference);
        var source = target == "library" ? "unit Candidate; end unit;" : "program Candidate; begin end; end program;";
        var sourcePath = directory.File("Candidate.fpas");
        File.WriteAllText(sourcePath, source);
        var result = await ProcessRunner.RunAsync([cli, "compile", sourcePath, "--output", output, "--target", target,
            "--debug", debug, "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", reference,
            "--diagnostics", "json"]);
        Assert.Equal(2, result.ExitCode);
        Assert.Contains("Reference paths must differ", result.Output + result.Error);
        Assert.Equal(original, File.ReadAllBytes(reference));
        if (extension != ".dll") Assert.False(File.Exists(output));
        Assert.Empty(Directory.GetDirectories(directory.Path, ".fpas-*"));
    }
}
