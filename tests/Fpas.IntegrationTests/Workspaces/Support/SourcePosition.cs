using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;
using Fpas.Workspaces.Queries;

namespace Fpas.IntegrationTests.Workspaces.Support;

internal static class SourcePosition
{
    internal static SourceSelection Select(TemporaryDirectory project, string file, string source, string marker)
    {
        var offset = source.IndexOf(marker, StringComparison.Ordinal);
        Assert.True(offset >= 0, marker);
        if (offset > 0 && source[offset] == '\n' && source[offset - 1] == '\r')
            offset--;
        var document = new SourceDocument(project.File(file), source);
        var location = document.Span(offset, 0);
        return new(location.Path, location.Line, location.Column);
    }
}
