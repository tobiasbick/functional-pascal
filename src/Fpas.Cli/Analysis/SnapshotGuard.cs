// [Documentation](../../../docs/pascal/program-structure/cli.md#require-a-snapshot)
using Fpas.Workspaces.Loading;
using System.CommandLine;

namespace Fpas.Cli.Analysis;

internal static class SnapshotGuard
{
    internal static Option<string> CreateOption()
    {
        var option = new Option<string>("--expected-snapshot") { Description = "Require the snapshotId from an earlier analysis of the same context." };
        option.RequireSha256("snapshotId");
        return option;
    }

    internal static void Verify(ParseResult parse, Option<string> option, string actual)
    {
        if (parse.GetValue(option) is { } expected && !expected.Equals(actual, StringComparison.OrdinalIgnoreCase))
            throw new ContextFailure("inputs_changed", "The current snapshot differs from --expected-snapshot. Refresh source positions before retrying.",
                expectedSnapshotId: expected, actualSnapshotId: actual);
    }
}
