using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Projects;

public sealed class InitTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Preview_is_reviewable_and_writes_nothing_and_repeated_creation_preserves_files()
    {
        using var directory = new TemporaryDirectory();
        var preview = await ProjectCliFixture.Cli(directory, "init", "project", "App", "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--dry-run", "--report", "json");
        Assert.Equal(0, preview.ExitCode); Assert.Equal("", preview.Error); Assert.False(Directory.Exists(directory.File("App")));
        using var json = JsonDocument.Parse(preview.Output);
        Assert.True(json.RootElement.GetProperty("dryRun").GetBoolean());
        Assert.Equal(3, json.RootElement.GetProperty("files").GetArrayLength());
        Assert.All(json.RootElement.GetProperty("files").EnumerateArray(), file => { Assert.Equal("create", file.GetProperty("state").GetString()); Assert.NotEmpty(file.GetProperty("content").GetString()!); });
        await ProjectCliFixture.Init(directory, sdk);
        var files = Directory.GetFiles(directory.File("App")).ToDictionary(file => file, File.GetLastWriteTimeUtc);
        var repeated = await ProjectCliFixture.Cli(directory, "init", "project", "App", "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--report", "json");
        Assert.Equal(0, repeated.ExitCode);
        using var repeatedJson = JsonDocument.Parse(repeated.Output);
        Assert.All(repeatedJson.RootElement.GetProperty("files").EnumerateArray(), file => Assert.Equal("unchanged", file.GetProperty("state").GetString()));
        foreach (var (file, stamp) in files) Assert.Equal(stamp, File.GetLastWriteTimeUtc(file));
    }

    [Fact]
    public async Task Conflicts_prevent_every_write_and_keep_existing_content()
    {
        using var directory = new TemporaryDirectory();
        Directory.CreateDirectory(directory.File("App"));
        File.WriteAllText(directory.File("App/main.fpas"), "existing user source");
        var result = await ProjectCliFixture.Cli(directory, "init", "project", "App", "--report", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output); Assert.Equal("conflict", json.RootElement.GetProperty("status").GetString());
        Assert.Equal("existing user source", File.ReadAllText(directory.File("App/main.fpas")));
        Assert.False(File.Exists(directory.File("App/App.fpasproj")));
    }

    [Fact]
    public async Task A_different_encoding_is_a_conflict_even_when_decoded_text_matches_the_template()
    {
        using var directory = new TemporaryDirectory();
        await ProjectCliFixture.Init(directory, sdk);
        var path = directory.File("App/main.fpas"); var text = File.ReadAllText(path);
        File.WriteAllText(path, text, System.Text.Encoding.Unicode);
        var original = File.ReadAllBytes(path);
        var result = await ProjectCliFixture.Cli(directory, "init", "project", "App", "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--report", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal(original, File.ReadAllBytes(path));
    }

    [Fact]
    public async Task The_packaged_CLI_defaults_templates_to_its_containing_SDK_version()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        var build = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json"); Assert.Equal(0, build.ExitCode);
        var cache = await ProcessRunner.RunAsync(["msbuild", project, "-nologo", "-getProperty:NuGetPackageRoot"]);
        Assert.Equal(0, cache.ExitCode);
        var cli = Path.Combine(cache.Output.Trim(), "fpas.net.sdk", sdk.Version.ToLowerInvariant(), "tools", "compiler", "Fpas.Cli.dll");
        Assert.True(File.Exists(cli));
        var init = await ProcessRunner.RunAsync([cli, "init", "library", directory.File("Packaged"), "--name", "Packaged", "--report", "json"]);
        Assert.True(init.ExitCode == 0, init.Output + init.Error);
        var created = XDocument.Load(directory.File("Packaged/Packaged.fpasproj"));
        Assert.Equal("Fpas.NET.Sdk/" + sdk.Version, (string?)created.Root!.Attribute("Sdk"));
    }

    [Theory]
    [InlineData("project", "program")]
    [InlineData("library", "Demo.Math")]
    public async Task Templates_build_using_the_selected_packaged_SDK_and_escape_keyword_names(string kind, string name)
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk, name, kind);
        var build = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json");
        Assert.True(build.ExitCode == 0, build.Output + build.Error); Assert.Equal("", build.Error);
        Assert.True(File.Exists(directory.File($"{name}/bin/Debug/net10.0/{name}.dll")));
        if (kind == "project")
        {
            var run = await ProjectCliFixture.Cli(directory, "run", project, "--no-build", "--report", "json");
            Assert.Equal(0, run.ExitCode);
            using var json = JsonDocument.Parse(run.Output); Assert.Equal("Hello from FPAS" + Environment.NewLine, json.RootElement.GetProperty("program").GetProperty("output").GetString());
        }
    }

    [Theory]
    [InlineData("init", "project", "App", "--sdk-version", "[1,2)")]
    [InlineData("init", "project", "App", "--name", "../Bad")]
    [InlineData("init", "project", "App", "--name", "3Bad")]
    [InlineData("init", "project", "App", "--name", "_")]
    [InlineData("init", "project", "App", "--sdk-feed", "ftp://example.com/packages")]
    [InlineData("init", "solution", "Demo", "--project", "missing.csproj")]
    [InlineData("init", "workspace", "Demo")]
    public async Task Invalid_input_has_structured_failures_and_no_side_effects(params string[] args)
    {
        using var directory = new TemporaryDirectory();
        var result = await ProjectCliFixture.Cli(directory, [.. args, "--report", "json"]);
        Assert.Equal(2, result.ExitCode);
        Assert.Empty(Directory.EnumerateFileSystemEntries(directory.Path));
        Assert.Equal("", result.Error); using var json = JsonDocument.Parse(result.Output); Assert.False(json.RootElement.GetProperty("success").GetBoolean());
    }
}
