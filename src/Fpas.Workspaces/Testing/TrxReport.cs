// [Documentation](../../../docs/pascal/tools/testing.md#results-and-process-ownership)
using Fpas.Workspaces.Loading;
using System.Globalization;
using System.Xml;
using System.Xml.Linq;

namespace Fpas.Workspaces.Testing;

/// <summary>Contains validated native test-result counts.</summary>
/// <param name="Total">Total leaf cases.</param>
/// <param name="Passed">Passed leaf cases.</param>
/// <param name="Failed">Failed leaf cases.</param>
/// <param name="Skipped">Leaf cases reported NotExecuted.</param>
public sealed record TestCounts(int Total, int Passed, int Failed, int Skipped);
/// <summary>Preserves one native TRX leaf-case result.</summary>
/// <param name="Name">Native case display name.</param>
/// <param name="FullyQualifiedName">Method identity from test definitions, when present.</param>
/// <param name="Outcome">Native outcome text.</param>
/// <param name="Duration">Native duration text, when present.</param>
/// <param name="Output">Case standard output.</param>
/// <param name="ErrorOutput">Case standard error.</param>
/// <param name="Error">Failure explanation.</param>
/// <param name="StackTrace">Native failure stack trace.</param>
public sealed record TestCaseResult(string Name, string? FullyQualifiedName, string Outcome, string? Duration, string? Output, string? ErrorOutput, string? Error, string? StackTrace)
{
    /// <summary>Gets the native TRX test identifier, when present.</summary>
    public string? Id { get; init; }
}
/// <summary>Preserves a native runner-level message.</summary>
/// <param name="Outcome">Native message severity or outcome.</param>
/// <param name="Text">Message text.</param>
public sealed record TestRunMessage(string Outcome, string Text);
/// <summary>Contains validated TRX cases, counters and runner-level failures.</summary>
/// <param name="Counts">Validated leaf-case counts.</param>
/// <param name="Tests">Native leaf-case results, excluding data-container duplicates.</param>
/// <param name="Outcome">Native summary outcome.</param>
/// <param name="Messages">Runner-level messages.</param>
/// <param name="Output">Summary standard output.</param>
/// <param name="ErrorOutput">Summary standard error.</param>
public sealed record TrxReport(TestCounts Counts, IReadOnlyList<TestCaseResult> Tests, string Outcome,
    IReadOnlyList<TestRunMessage> Messages, string? Output, string? ErrorOutput)
{
    /// <summary>Gets whether every case passed or skipped and the native runner completed without errors.</summary>
    public bool Success => Counts.Failed == 0 && (long)Counts.Passed + Counts.Skipped == Counts.Total &&
        (Outcome is "Completed" or "Passed") && !Messages.Any(message => message.Outcome == "Error");
    /// <summary>Reads bounded native TRX XML and rejects inconsistent counters or missing cases.</summary>
    /// <param name="path">Native results.trx artifact path.</param>
    /// <param name="token">Cancels case traversal.</param>
    /// <returns>The validated native report.</returns>
    public static TrxReport Read(string path, CancellationToken token)
    {
        token.ThrowIfCancellationRequested();
        if (!File.Exists(path)) throw new ContextFailure("results_missing", "The test runner did not produce the requested TRX report.");
        var settings = new XmlReaderSettings { DtdProcessing = DtdProcessing.Prohibit, XmlResolver = null, MaxCharactersInDocument = 16_777_216 };
        using var reader = XmlReader.Create(path, settings);
        var document = XDocument.Load(reader);
        XNamespace ns = "http://microsoft.com/schemas/VisualStudio/TeamTest/2010";
        if (document.Root?.Name != ns + "TestRun") throw new ContextFailure("results_invalid", "Expected a VSTest TRX document.");
        var summary = document.Root.Element(ns + "ResultSummary");
        var counters = summary?.Element(ns + "Counters") ?? throw new ContextFailure("results_invalid", "TRX counters are missing.");
        int Count(string name) => int.TryParse((string?)counters.Attribute(name), NumberStyles.None, CultureInfo.InvariantCulture, out var value) ? value :
            throw new ContextFailure("results_invalid", "Invalid TRX counter: " + name);
        var definitions = document.Root.Element(ns + "TestDefinitions")?.Elements(ns + "UnitTest").ToDictionary(
            element => (string)element.Attribute("id")!, element => element.Element(ns + "TestMethod") is { } method ?
                (string?)method.Attribute("className") + "." + (string?)method.Attribute("name") : null) ?? [];
        var tests = new List<TestCaseResult>();
        foreach (var result in document.Root.Element(ns + "Results")?.Descendants(ns + "UnitTestResult") ?? [])
        {
            token.ThrowIfCancellationRequested();
            if (result.Descendants(ns + "UnitTestResult").Any()) continue;
            var output = result.Element(ns + "Output"); var error = output?.Element(ns + "ErrorInfo");
            var id = (string?)result.Attribute("testId");
            definitions.TryGetValue(id ?? "", out var name);
            tests.Add(new((string?)result.Attribute("testName") ?? "", name, (string?)result.Attribute("outcome") ?? "Unknown",
                (string?)result.Attribute("duration"), output?.Element(ns + "StdOut")?.Value, output?.Element(ns + "StdErr")?.Value, error?.Element(ns + "Message")?.Value,
                error?.Element(ns + "StackTrace")?.Value) { Id = id });
        }
        // VSTest leaves several legacy summary counters at zero, including notExecuted.
        var counts = new TestCounts(Count("total"), Count("passed"), Count("failed"), tests.Count(test => test.Outcome == "NotExecuted"));
        if (tests.Count != counts.Total || counts.Passed != tests.Count(test => test.Outcome == "Passed") ||
            counts.Failed != tests.Count(test => test.Outcome == "Failed") || (long)counts.Passed + counts.Failed + counts.Skipped > counts.Total)
            throw new ContextFailure("results_invalid", "Inconsistent TRX counters or missing case results.");
        var outcome = (string?)summary!.Attribute("outcome") ?? throw new ContextFailure("results_invalid", "TRX summary outcome is missing.");
        var messages = summary.Element(ns + "RunInfos")?.Elements(ns + "RunInfo")
            .Select(info => new TestRunMessage((string?)info.Attribute("outcome") ?? "Unknown", info.Element(ns + "Text")?.Value ?? "")).ToArray() ?? [];
        return new(counts, tests.AsReadOnly(), outcome, Array.AsReadOnly(messages),
            summary.Element(ns + "Output")?.Element(ns + "StdOut")?.Value, summary.Element(ns + "Output")?.Element(ns + "StdErr")?.Value);
    }
}
