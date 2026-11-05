using Fpas.Compiler.Formatting;
using Fpas.Tests.Support;
using System.Text;
using System.Text.Json;

namespace Fpas.IntegrationTests.Formatting;

public sealed class FormatCliTests
{
    private const string Source = "program Demo;uses System.Console as Console;begin Console.WriteLine('ok');end;end program;";
    private static string Cli => TestEnvironment.Metadata("FpasCliPath");
    internal static Task<ProcessResult> Run(TemporaryDirectory directory, params string[] args)
        => ProcessRunner.RunAsync([Cli, "fmt", .. args], workingDirectory: directory.Path);

    [Fact]
    public async Task Check_list_stdout_write_and_no_change_preserve_their_stream_and_file_contracts()
    {
        using var directory = new TemporaryDirectory();
        var path = directory.File("main source 😀.fpas"); File.WriteAllText(path, Source);
        var expected = FpasFormatter.Format(new(path, Source)).Text;
        var check = await Run(directory, path, "--check", "--report", "json");
        Assert.Equal(2, check.ExitCode); Assert.Empty(check.Error);
        using (var report = JsonDocument.Parse(check.Output))
        {
            Assert.False(report.RootElement.GetProperty("success").GetBoolean());
            Assert.True(Assert.Single(report.RootElement.GetProperty("files").EnumerateArray()).GetProperty("changed").GetBoolean());
        }
        Assert.Equal(Source, File.ReadAllText(path));
        var list = await Run(directory, path, "--check", "--list");
        Assert.Equal(2, list.ExitCode); Assert.Equal(path + Environment.NewLine, list.Output); Assert.Empty(list.Error);
        var stdout = await Run(directory, path, "--stdout");
        Assert.Equal(0, stdout.ExitCode); Assert.Equal(expected, stdout.Output); Assert.Empty(stdout.Error);
        Assert.Equal(Source, File.ReadAllText(path));
        var write = await Run(directory, path, "--report", "json"); Assert.Equal(0, write.ExitCode);
        using (var report = JsonDocument.Parse(write.Output))
            Assert.True(Assert.Single(report.RootElement.GetProperty("files").EnumerateArray()).GetProperty("written").GetBoolean());
        Assert.Equal(expected, File.ReadAllText(path));
        var modified = File.GetLastWriteTimeUtc(path);
        var again = await Run(directory, path, path, "--report", "json"); Assert.Equal(0, again.ExitCode);
        using (var report = JsonDocument.Parse(again.Output)) Assert.Single(report.RootElement.GetProperty("files").EnumerateArray());
        Assert.Equal(modified, File.GetLastWriteTimeUtc(path));
        var clean = await Run(directory, path, "--check"); Assert.Equal(0, clean.ExitCode); Assert.Empty(clean.Output + clean.Error);
    }

    [Fact]
    public async Task Every_input_is_parsed_before_any_source_is_written()
    {
        using var directory = new TemporaryDirectory();
        var good = directory.File("a.fpas"); var bad = directory.File("z.fpas");
        File.WriteAllText(good, Source); File.WriteAllText(bad, "program Broken;begin");
        var result = await Run(directory, directory.Path, "--report", "json");
        Assert.Equal(1, result.ExitCode); Assert.Empty(result.Error);
        Assert.Equal(Source, File.ReadAllText(good)); Assert.Equal("program Broken;begin", File.ReadAllText(bad));
        using var report = JsonDocument.Parse(result.Output);
        Assert.False(report.RootElement.GetProperty("success").GetBoolean());
        Assert.NotEmpty(report.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.DoesNotContain(report.RootElement.GetProperty("files").EnumerateArray(), file => file.GetProperty("written").GetBoolean());
    }

    [Fact]
    public async Task Directories_are_recursive_and_skip_generated_and_linked_inputs()
    {
        using var directory = new TemporaryDirectory();
        foreach (var name in new[] { "a.fpas", "nested/b.fpas", "obj/generated.fpas", "bin/copied.fpas", ".temp-data/scratch.fpas", ".git/hidden.fpas", "node_modules/foreign.fpas" })
        {
            var path = directory.File(name); Directory.CreateDirectory(Path.GetDirectoryName(path)!); File.WriteAllText(path, Source);
        }
        if (!OperatingSystem.IsWindows()) File.CreateSymbolicLink(directory.File("link.fpas"), directory.File("a.fpas"));
        var result = await Run(directory, directory.Path, directory.File("nested"), "--check", "--list");
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error);
        Assert.Equal(new[] { directory.File("a.fpas"), directory.File("nested/b.fpas") }, result.Output.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries));
    }

    [Fact]
    public async Task Explicit_links_and_links_in_parent_paths_are_rejected_without_touching_targets()
    {
        if (OperatingSystem.IsWindows()) return;
        using var directory = new TemporaryDirectory();
        Directory.CreateDirectory(directory.File("sources"));
        var target = directory.File("sources/main.fpas"); File.WriteAllText(target, Source);
        File.CreateSymbolicLink(directory.File("link.fpas"), target);
        Directory.CreateSymbolicLink(directory.File("linked"), directory.File("sources"));
        foreach (var path in new[] { directory.File("link.fpas"), directory.File("linked/main.fpas") })
        {
            var result = await Run(directory, path, "--report", "json"); Assert.Equal(2, result.ExitCode);
            using var report = JsonDocument.Parse(result.Output);
            Assert.Equal("invalid_argument", report.RootElement.GetProperty("error").GetProperty("code").GetString());
            Assert.Equal(Source, File.ReadAllText(target));
        }
    }

    [Fact]
    public async Task UTF8_BOM_and_line_endings_normalize_once_and_file_permissions_are_preserved()
    {
        using var directory = new TemporaryDirectory();
        var path = directory.File("main.fpas");
        File.WriteAllText(path, FpasFormatter.Format(new(path, Source)).Text!.Replace("\n", "\r\n"), new UTF8Encoding(true));
        UnixFileMode? permissions = OperatingSystem.IsWindows() ? null : UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute;
        if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(path, permissions!.Value);
        var result = await Run(directory, path); Assert.Equal(0, result.ExitCode); Assert.Empty(result.Output + result.Error);
        Assert.DoesNotContain('\r', File.ReadAllText(path)); Assert.NotEqual((byte)0xEF, File.ReadAllBytes(path)[0]);
        if (!OperatingSystem.IsWindows()) Assert.Equal(permissions, File.GetUnixFileMode(path));
        Assert.Empty(Directory.GetFiles(directory.Path, ".fpas-fmt-*"));
    }

    [Theory]
    [InlineData("--list")]
    [InlineData("--check", "--stdout")]
    [InlineData("--stdout")]
    [InlineData("--timeout", "0")]
    [InlineData("--unknown")]
    [InlineData("--configuration")]
    public async Task Invalid_options_return_one_JSON_document(params string[] options)
    {
        using var directory = new TemporaryDirectory(); File.WriteAllText(directory.File("main.fpas"), Source);
        var result = await Run(directory, ["main.fpas", "--report", "json", .. options]);
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid_argument", report.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Fact]
    public async Task UTF8_read_errors_and_multiple_stdout_sources_leave_all_sources_untouched()
    {
        using var directory = new TemporaryDirectory();
        File.WriteAllText(directory.File("good.fpas"), Source);
        File.WriteAllBytes(directory.File("bad.fpas"), [0xFF, 0xFF]);
        var result = await Run(directory, directory.Path, "--report", "json"); Assert.Equal(1, result.ExitCode);
        using (var report = JsonDocument.Parse(result.Output)) Assert.Equal("format_failed", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        var multiple = await Run(directory, directory.Path, "--stdout"); Assert.Equal(2, multiple.ExitCode); Assert.Empty(multiple.Output);
        Assert.Equal(Source, File.ReadAllText(directory.File("good.fpas")));
    }

    [Fact]
    public async Task Literal_dash_and_at_paths_require_no_response_file_expansion()
    {
        using var directory = new TemporaryDirectory();
        File.WriteAllText(directory.File("--source.fpas"), Source); File.WriteAllText(directory.File("@source.fpas"), Source);
        var result = await Run(directory, "--check", "--list", "--", "--source.fpas", "@source.fpas");
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error);
        Assert.Equal(new[] { directory.File("--source.fpas"), directory.File("@source.fpas") }, result.Output.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries));
    }

    [Fact]
    public async Task A_read_only_source_is_preserved_and_no_staging_file_is_left()
    {
        using var directory = new TemporaryDirectory();
        var path = directory.File("main.fpas"); File.WriteAllText(path, Source);
        var attributes = File.GetAttributes(path);
        try
        {
            File.SetAttributes(path, attributes | FileAttributes.ReadOnly);
            var result = await Run(directory, path, "--report", "json"); Assert.Equal(1, result.ExitCode);
            Assert.Equal(Source, File.ReadAllText(path));
            using var report = JsonDocument.Parse(result.Output);
            Assert.False(Assert.Single(report.RootElement.GetProperty("files").EnumerateArray()).GetProperty("written").GetBoolean());
            Assert.Empty(Directory.GetFiles(directory.Path, ".fpas-fmt-*"));
        }
        finally { File.SetAttributes(path, attributes); }
    }

    [Fact]
    public async Task Malformed_solutions_and_unavailable_dotnet_hosts_produce_structured_failures()
    {
        using var directory = new TemporaryDirectory();
        File.WriteAllText(directory.File("Broken.slnx"), "not XML");
        var broken = await Run(directory, "Broken.slnx", "--report", "json"); Assert.NotEqual(0, broken.ExitCode); Assert.Empty(broken.Error);
        using (var report = JsonDocument.Parse(broken.Output)) Assert.False(report.RootElement.GetProperty("success").GetBoolean());
        File.WriteAllText(directory.File("Project.fpasproj"), "<Project />");
        var host = await ProcessRunner.RunAsync([Cli, "fmt", "Project.fpasproj", "--report", "json"], workingDirectory: directory.Path,
            environment: new Dictionary<string, string?> { ["DOTNET_HOST_PATH"] = directory.File("missing-dotnet") });
        Assert.Equal(1, host.ExitCode); Assert.Empty(host.Error);
        using (var report = JsonDocument.Parse(host.Output)) Assert.Equal("format_failed", report.RootElement.GetProperty("error").GetProperty("code").GetString());
    }
}
