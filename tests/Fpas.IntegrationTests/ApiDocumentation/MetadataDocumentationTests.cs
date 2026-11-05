using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Documentation;
using Fpas.Workspaces.Loading;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class MetadataDocumentationTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Public_CLR_signatures_match_real_CSharp_XML_ids_including_nested_generics_and_indexers()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var analysis = await DocumentationFixture.Analyze(directory);
        var query = new DocumentationQuery(analysis);
        var result = query.Search(new("Lookup.", "prefix", Assembly: "Shared", Limit: 100));
        Assert.Equal("ok", result.Status); Assert.False(result.Truncated);
        var native = XDocument.Load(directory.File("Shared/bin/Debug/net10.0/Shared.xml")).Descendants("member")
            .Where(member => ((string)member.Attribute("name")!).Contains("Lookup.", StringComparison.Ordinal)).ToArray();
        Assert.NotEmpty(native);
        foreach (var member in native)
        {
            Assert.Contains((string?)member.Attribute("name"), result.Entries.Select(entry => entry.DocumentationId));
            var entry = Assert.Single(result.Entries, entry => entry.DocumentationId == (string?)member.Attribute("name"));
            Assert.Equal("available", entry.Documentation.Availability);
            Assert.True(XNode.DeepEquals(member, XElement.Parse(entry.Documentation.Content!)), entry.DocumentationId);
        }
        Assert.DoesNotContain(result.Entries, entry => entry.Name is "get_Item" or "set_Item" or "Hidden" or "HiddenByOwner" or "value__");
        Assert.Contains(result.Entries, entry => entry.Kind == "constructor");
        Assert.Contains(result.Entries, entry => entry.Kind == "event");
        Assert.Contains(result.Entries, entry => entry.Name == "Limit" && entry.Signature.Contains("42", StringComparison.Ordinal));
        Assert.Contains(result.Entries, entry => entry.Name == "Shapes" && entry.Signature.Contains("params System.Object[] more", StringComparison.Ordinal));
        Assert.Contains(result.Entries, entry => entry.Name == "Echo" && entry.Signature.Contains("ref ", StringComparison.Ordinal) && entry.Signature.Contains("out ", StringComparison.Ordinal) && entry.Signature.Contains("in ", StringComparison.Ordinal));
        var property = Assert.Single(result.Entries, entry => entry.Name == "ReadOnly"); Assert.DoesNotContain("set;", property.Signature);
        query.VerifyInputs(); await analysis.Context.VerifyInputsAsync();
        Assert.False(File.Exists(directory.File("initializer-executed"))); Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
    }

    [Fact]
    public async Task Source_docs_use_existing_inspection_and_exact_type_lookup_includes_direct_members()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var analysis = await DocumentationFixture.Analyze(directory); var query = new DocumentationQuery(analysis);
        var result = query.Search(new("Demo.Bridge.Message"));
        var source = Assert.Single(result.Entries); Assert.Equal("source", source.Origin); Assert.Equal("fpas", source.SignatureFormat);
        Assert.Equal("available", source.Documentation.Availability); Assert.Contains("## Parameters\n- `Value`: Text to decorate.", source.Documentation.Content);
        Assert.DoesNotContain("//", source.Documentation.Content); Assert.Contains("Value", source.Signature);
        var point = query.Search(new("Demo.Bridge.Point"));
        Assert.Equal(new[] { "Point", "Read", "X" }, point.Entries.Select(entry => entry.Name).Order(StringComparer.Ordinal));
        var native = query.Search(new("Lookup.Api", Limit: 100));
        Assert.Contains(native.Entries, entry => entry.Name == "Echo"); Assert.Contains(native.Entries, entry => entry.Kind == "type");
        Assert.Contains(native.Entries, entry => entry.Name == "Nested");
        Assert.DoesNotContain(native.Entries, entry => entry.Name == "Combine");
        var unknown = query.Search(new("DoesNotExist")); Assert.Equal("not_found", unknown.Status); Assert.Empty(unknown.Entries);
    }

    [Fact]
    public async Task Paging_and_XML_changes_have_explicit_consistency_and_missing_documentation_states()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var analysis = await DocumentationFixture.Analyze(directory); var query = new DocumentationQuery(analysis);
        var first = query.Search(new("Echo", Limit: 1)); var second = query.Search(new("Echo", Limit: 1, Offset: 1));
        Assert.True(first.Truncated); Assert.Equal(1, first.NextOffset); Assert.Equal(first.SnapshotId, second.SnapshotId);
        Assert.NotEqual(first.Entries[0].DocumentationId, second.Entries[0].DocumentationId);
        var path = Assert.Single(first.Entries).Documentation.Path!;
        File.AppendAllText(path, "\n"); Assert.Throws<ContextFailure>(() => query.VerifyInputs());
        var changed = query.Search(new("Echo", Limit: 1)); Assert.NotEqual(first.SnapshotId, changed.SnapshotId);
        File.Delete(path);
        var missing = query.Search(new("Echo", Assembly: "Shared", Limit: 100));
        Assert.All(missing.Entries, entry => Assert.Equal("file_missing", entry.Documentation.Availability));
        File.WriteAllText(path, "<!DOCTYPE doc SYSTEM 'file:///not-read'><doc><members /></doc>");
        Assert.Throws<ContextFailure>(() => query.Search(new("Echo", Assembly: "Shared")));
    }
}
