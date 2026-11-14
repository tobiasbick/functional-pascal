using System.Xml.Linq;
using Fpas.Processes;

namespace Fpas.IntegrationTests.Build;

internal static class WorkflowProjects
{
    // Build the FPAS dependencies with the same isolated SDK as the consumer.
    internal static void Copy(string repository, string destination, string sdkVersion)
    {
        foreach (var name in new[] { "Fpas.Workflows", "Fpas.Processes.Workflows" })
        {
            var source = Path.Combine(repository, "src", name);
            var target = Path.Combine(destination, name);
            Directory.CreateDirectory(target);
            foreach (var file in Directory.GetFiles(source, "*.fpas"))
                File.Copy(file, Path.Combine(target, Path.GetFileName(file)));

            var project = XDocument.Load(Path.Combine(source, name + ".fpasproj"));
            project.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + sdkVersion);
            var backend = project.Descendants("ProjectReference").SingleOrDefault(item =>
                Path.GetFileName((string?)item.Attribute("Include")) == "Fpas.Processes.csproj");
            backend?.ReplaceWith(new XElement("Reference", new XAttribute("Include", "Fpas.Processes"),
                new XElement("HintPath", typeof(OwnedProcess).Assembly.Location)));
            project.Save(Path.Combine(target, name + ".fpasproj"));
        }
    }
}
