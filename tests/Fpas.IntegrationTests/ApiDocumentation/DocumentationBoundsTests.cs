using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Documentation;
using Fpas.Workspaces.Loading;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class DocumentationBoundsTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Text_budgets_missing_members_wrong_assemblies_and_XML_entities_have_explicit_outcomes()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var analysis = await DocumentationFixture.Analyze(directory); var query = new DocumentationQuery(analysis);
        var entry = Assert.Single(query.Search(new("Lookup.Api.Limit")).Entries); var path = entry.Documentation.Path!;
        var xml = XDocument.Load(path);
        var member = xml.Descendants("member").Single(element => (string?)element.Attribute("name") == entry.DocumentationId);
        member.ReplaceNodes(new XElement("summary", new string('x', 70_000))); xml.Save(path);
        var large = query.Search(new("Lookup.Api.Limit"));
        Assert.True(large.Truncated); Assert.Equal(1, large.OmittedDocumentationCount);
        Assert.Equal("text_budget", Assert.Single(large.Entries).Documentation.Availability); Assert.Null(large.Entries[0].Documentation.Content);
        member.Remove(); xml.Save(path);
        Assert.Equal("member_missing", Assert.Single(query.Search(new("Lookup.Api.Limit")).Entries).Documentation.Availability);
        xml.Root!.Element("assembly")!.Element("name")!.Value = "WrongAssembly"; xml.Save(path);
        Assert.Equal("documentation_invalid", Assert.Throws<ContextFailure>(() => query.Search(new("Lookup.Api.Limit"))).Code);
        var entity = directory.File("entity.txt"); File.WriteAllText(entity, "External entity must not become documentation.");
        File.WriteAllText(path, $"<!DOCTYPE doc [<!ENTITY unsafe SYSTEM '{new Uri(entity).AbsoluteUri}'>]><doc><members><member name='F:Lookup.Api`1.Limit'><summary>&unsafe;</summary></member></members></doc>");
        Assert.Equal("documentation_invalid", Assert.Throws<ContextFailure>(() => query.Search(new("Lookup.Api.Limit"))).Code);
    }

    [Fact]
    public async Task Native_markup_is_preserved_and_plain_text_keeps_cross_references()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var analysis = await DocumentationFixture.Analyze(directory); var query = new DocumentationQuery(analysis);
        var result = query.Search(new("Lookup.Api", Limit: 100));
        var type = Assert.Single(result.Entries, entry => entry.Name == "Api");
        Assert.Contains("<see", type.Documentation.Content); Assert.Contains("System.String", DocumentationDisplay.Text(type.Documentation));
        Assert.Contains("<XML>", DocumentationDisplay.Text(type.Documentation));
        Assert.Contains("null", DocumentationDisplay.Text(type.Documentation));
        using var cancelled = new CancellationTokenSource(); cancelled.Cancel();
        Assert.Throws<OperationCanceledException>(() => query.Search(new("Echo"), cancelled.Token));
    }
}
