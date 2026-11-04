using Fpas.Tests.Support;
using Fpas.Workspaces.Queries;

namespace Fpas.IntegrationTests.Workspaces.Support;

internal static class SourcePosition
{
    internal static SourceSelection Select(TemporaryDirectory project, string file, string source, string marker)
    {
        var offset = source.IndexOf(marker, StringComparison.Ordinal); Assert.True(offset >= 0, marker);
        var prefix = source[..offset].Split('\n');
        return new(project.File(file), prefix.Length, prefix[^1].Length + 1);
    }
}
