// [Documentation](../../docs/pascal/language/basics/lexical-rules.md)
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler;

/// <summary>Canonical source spelling for generated qualified names.</summary>
public static class FpasIdentifiers
{
    /// <summary>Spells a source name with its type arguments or parameters.</summary>
    /// <param name="name">The already spelled source name, or an empty prefix for a routine suffix.</param>
    /// <param name="arguments">Source spellings of the type arguments or parameters.</param>
    /// <param name="groupBeforeSuffix">Groups a single-argument application before a member or nullable suffix.</param>
    /// <returns>The completed source spelling.</returns>
    public static string ApplyTypeArguments(string name, IReadOnlyList<string> arguments, bool groupBeforeSuffix = false)
        => arguments.Count switch
        {
            0 => name,
            1 when groupBeforeSuffix => "(" + name + " of " + arguments[0] + ")",
            1 => name + " of " + arguments[0],
            _ => name + " of (" + string.Join(", ", arguments) + ")"
        };

    /// <summary>Spells a dot-qualified name, escaping keyword parts with '@'.</summary>
    /// <param name="name">A qualified name of ASCII identifier parts separated by '.'.</param>
    /// <returns>The source spelling of <paramref name="name"/>.</returns>
    /// <exception cref="ArgumentException"><paramref name="name"/> is not a qualified identifier.</exception>
    public static string SpellQualified(string name)
    {
        ArgumentNullException.ThrowIfNull(name);
        var parts = name.Split('.');
        if (parts.Any(part => part.Length == 0 || part == "_" || !(char.IsAsciiLetter(part[0]) || part[0] == '_') || part.Any(character => !char.IsAsciiLetterOrDigit(character) && character != '_')))
            throw new ArgumentException("A qualified identifier is required.", nameof(name));
        return string.Join('.', parts.Select(part => Lexer.IsKeyword(part) ? "@" + part : part));
    }
}
