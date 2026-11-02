namespace Fpas.Build.Tasks;

internal static class DocumentationOutputs
{
    internal static StringComparer Comparer => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    internal static string[] Paths(string intermediate, string destination, string conventional)
        => new[] { intermediate, destination, conventional }.Where(path => !string.IsNullOrEmpty(path)).Select(Path.GetFullPath).Distinct(Comparer).ToArray();

    internal static void Publish(IReadOnlyList<string> documents, IEnumerable<string> oldDocuments)
    {
        foreach (var path in documents.Skip(1))
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            var temporary = path + "." + Guid.NewGuid().ToString("N") + ".tmp";
            try { File.Copy(documents[0], temporary); File.Move(temporary, path, overwrite: true); }
            finally { File.Delete(temporary); }
        }
        foreach (var obsolete in oldDocuments.Except(documents, Comparer)) File.Delete(obsolete);
    }
}
