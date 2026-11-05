using System.Globalization;
using System.Text;

namespace Fpas.Workspaces.Processes;

public static class MsBuildSyntax
{
    public static string Escape(string value)
    {
        var result = new StringBuilder();
        foreach (var character in value)
            result.Append("%*?@$();,'\"".Contains(character) ? "%" + ((int)character).ToString("X2", CultureInfo.InvariantCulture) : character);
        return result.ToString();
    }
}
