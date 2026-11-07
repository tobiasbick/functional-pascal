using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Text;
using System.Text.Json;

namespace Fpas.IntegrationTests.Testing;

public sealed class TestReportLimitTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Oversized_native_results_keep_artifacts_and_return_one_bounded_failure()
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, "nunit");
        File.WriteAllText(directory.File("Tests/Tests.fpas"), """
            unit LargeOutput;
            uses NUnit.Framework as N, System as S;
            [N.TestFixtureAttribute]
            public type Tests = record
              [N.TestAttribute]
              public procedure Output();
              begin
                const Text: string := 'x';
                S.Console.WriteLine(Text.PadRight(6_000_000, #120));
              end procedure;
            end record;
            end unit;
            """);
        var result = await ProjectCliFixture.Cli(directory, "test", project, "--report", "json");
        Assert.Equal(1, result.ExitCode); Assert.Empty(result.Error); Assert.InRange(Encoding.UTF8.GetByteCount(result.Output), 1, 4_194_304);
        using var report = JsonDocument.Parse(result.Output);
        Assert.False(report.RootElement.GetProperty("success").GetBoolean());
        Assert.Equal("report_limit", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        var folder = report.RootElement.GetProperty("resultsDirectory").GetString()!;
        Assert.Single(Directory.GetFiles(folder, "results.trx", SearchOption.AllDirectories));
    }
}
