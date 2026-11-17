using System.Globalization;
using System.Text;

namespace Fpas.Workspaces.Processes;

/// <summary>Escapes literal values passed through MSBuild property and item syntax.</summary>
public static class MsBuildSyntax
{
    /// <summary>Percent-escapes characters MSBuild interprets as expressions or separators.</summary>
    /// <param name="value">Literal property or item value.</param>
    /// <returns>The value encoded for MSBuild parsing.</returns>
    public static string Escape(string value)
    {
        var result = new StringBuilder();
        foreach (var character in value)
            result.Append("%*?@$();,'\"".Contains(character) ? "%" + ((int)character).ToString("X2", CultureInfo.InvariantCulture) : character);
        return result.ToString();
    }
}
