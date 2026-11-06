using Fpas.Tests.Support;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Loading;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class LiveDocumentTests
{
    [Theory]
    [InlineData("\r\n")]
    [InlineData("\n")]
    [InlineData("\r")]
    public async Task Ordered_UTF16_changes_are_atomic_and_versions_are_strict(string newline)
    {
        using var directory = new TemporaryDirectory();
        await using var workspace = new LiveWorkspace(new(directory.File("Missing.fpasproj")));
        var path = directory.File("unsaved.fpas"); var original = "// 😀" + newline + "unit Loose;" + newline + "end unit;";
        workspace.Open(path, original, 7);
        var captured = workspace.OpenDocuments;
        workspace.Change(path, 8, [new("WideName", new(new(2, 6), new(2, 11))), new(" // note", new(new(2, 15), new(2, 15)))]);
        Assert.Equal(original, captured.Document(path).Text);
        Assert.Equal(original.Replace("Loose;", "WideName; // note", StringComparison.Ordinal), workspace.OpenDocuments.Document(path).Text);
        Assert.Equal(8, workspace.OpenDocuments.Document(path).Version);
        var revision = workspace.Revision;
        Assert.Equal("invalid_version", Assert.Throws<ContextFailure>(() => workspace.Change(path, 8, [new("bad")])).Code);
        Assert.Equal("invalid_change", Assert.Throws<ContextFailure>(() => workspace.Change(path, 9, [new("valid replacement"), new("x", new(new(99, 1), new(99, 1)))] )).Code);
        Assert.Equal(revision, workspace.Revision); Assert.Equal(8, workspace.OpenDocuments.Document(path).Version);
        workspace.Save(path); Assert.Equal(8, workspace.OpenDocuments.Document(path).Version);
        workspace.Close(path); Assert.Empty(workspace.OpenDocuments.Documents); Assert.False(File.Exists(path));
        Assert.Equal("invalid_document", Assert.Throws<ContextFailure>(() => workspace.Save(path)).Code);
    }

    [Fact]
    public async Task Document_and_session_limits_fail_without_mutating_open_text()
    {
        using var directory = new TemporaryDirectory();
        await using var workspace = new LiveWorkspace(new(directory.File("Missing.fpasproj")), new(DocumentBytes: 12, OpenBytes: 16));
        var a = directory.File("a.fpas"); var b = directory.File("b.fpas");
        workspace.Open(a, "12345678", 1); workspace.Open(b, "😀😀", 1); var revision = workspace.Revision;
        Assert.Equal("workspace_limit", Assert.Throws<ContextFailure>(() => workspace.Change(a, 2, [new("123456789")])).Code);
        Assert.Equal("workspace_limit", Assert.Throws<ContextFailure>(() => workspace.Open(directory.File("huge.fpas"), "😀😀😀😀", 1)).Code);
        Assert.Equal(revision, workspace.Revision); Assert.Equal("12345678", workspace.OpenDocuments.Document(a).Text);
        workspace.Close(b); workspace.Change(a, 2, [new("123456789")]); Assert.Equal(2, workspace.OpenDocuments.Document(a).Version);
    }
}
