using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Testing;

namespace Fpas.IntegrationTests.Testing;

public sealed class TrxReportTests
{
    [Fact]
    public void Skips_come_from_native_results_and_data_containers_are_not_double_counted()
    {
        using var directory = new TemporaryDirectory();
        var file = directory.File("result.trx");
        File.WriteAllText(file, """
            <TestRun xmlns="http://microsoft.com/schemas/VisualStudio/TeamTest/2010">
              <Results>
                <UnitTestResult testName="data parent" outcome="Passed"><InnerResults>
                  <UnitTestResult testName="row" outcome="Passed" />
                </InnerResults></UnitTestResult>
                <UnitTestResult testName="ignored" outcome="NotExecuted" />
              </Results>
              <ResultSummary outcome="Completed"><Counters total="2" passed="1" failed="0" notExecuted="0" /></ResultSummary>
            </TestRun>
            """);
        var result = TrxReport.Read(file, CancellationToken.None);
        Assert.True(result.Success); Assert.Equal(new TestCounts(2, 1, 0, 1), result.Counts);
        Assert.Equal(new[] { "row", "ignored" }, result.Tests.Select(test => test.Name));
    }

    [Theory]
    [InlineData("<TestRun />")]
    [InlineData("<TestRun xmlns='http://microsoft.com/schemas/VisualStudio/TeamTest/2010'><ResultSummary outcome='Completed'><Counters total='-1' passed='0' failed='0'/></ResultSummary></TestRun>")]
    [InlineData("<TestRun xmlns='http://microsoft.com/schemas/VisualStudio/TeamTest/2010'><ResultSummary outcome='Completed'><Counters total='1' passed='2' failed='0'/></ResultSummary></TestRun>")]
    [InlineData("<TestRun xmlns='http://microsoft.com/schemas/VisualStudio/TeamTest/2010'><ResultSummary outcome='Completed'><Counters total='1' passed='1' failed='0'/></ResultSummary></TestRun>")]
    public void Missing_or_inconsistent_native_results_are_failures(string xml)
    {
        using var directory = new TemporaryDirectory(); var file = directory.File("bad.trx"); File.WriteAllText(file, xml);
        Assert.Throws<ContextFailure>(() => TrxReport.Read(file, CancellationToken.None));
    }

    [Theory]
    [InlineData("Completed", "Warning", true)]
    [InlineData("Completed", "Error", false)]
    [InlineData("Aborted", "Warning", false)]
    public void Run_messages_and_host_failures_are_preserved_even_without_failed_cases(string outcome, string severity, bool success)
    {
        using var directory = new TemporaryDirectory(); var file = directory.File("messages.trx");
        File.WriteAllText(file, $$"""
            <TestRun xmlns="http://microsoft.com/schemas/VisualStudio/TeamTest/2010">
              <ResultSummary outcome="{{outcome}}">
                <Counters total="0" passed="0" failed="0" />
                <Output><StdOut>native stdout</StdOut><StdErr>native stderr</StdErr></Output>
                <RunInfos><RunInfo outcome="{{severity}}"><Text>native message</Text></RunInfo></RunInfos>
              </ResultSummary>
            </TestRun>
            """);
        var result = TrxReport.Read(file, CancellationToken.None);
        Assert.Equal(success, result.Success); Assert.Equal("native stdout", result.Output); Assert.Equal("native stderr", result.ErrorOutput);
        Assert.Equal(new TestRunMessage(severity, "native message"), Assert.Single(result.Messages));
    }
}
