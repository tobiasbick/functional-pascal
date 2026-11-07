using System.Text;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Evaluation;

internal static class ExpressionLexer
{
    private static readonly HashSet<string> Keywords = new(("array async await begin break case catch const continue discard distinct div do downto else elsif end ensures enum extends false finally for function if implements in interface is mod new not null of or out params private procedure program public raise range record ref repeat return requires shl shr static subscribe then to true try type union unit unsubscribe until using var when where while with and xor as uses class struct cast typeof unchecked").Split(' '), StringComparer.Ordinal);

    internal static ExpressionToken[] Read(string source)
    {
        var result = new List<ExpressionToken>(); var position = 0;
        while (position < source.Length)
        {
            if (char.IsWhiteSpace(source[position])) { position++; continue; }
            if (source.AsSpan(position).StartsWith("//")) { while (position < source.Length && source[position] is not ('\n' or '\r')) position++; continue; }
            var start = position; var current = source[position++]; string kind, text;
            if (current == '\'')
            {
                var buffer = new StringBuilder(); var closed = false;
                while (position < source.Length)
                {
                    var character = source[position++];
                    if (character == '\'')
                    { if (position < source.Length && source[position] == '\'') position++; else { closed = true; break; } }
                    if (character == '\r')
                    {
                        if (position < source.Length && source[position] == '\n') position++;
                        character = '\n';
                    }
                    buffer.Append(character);
                }
                if (!closed) throw Invalid(start, "Unterminated string literal.");
                kind = "literal"; text = "'" + buffer;
            }
            else if (char.IsAsciiLetter(current) || current is '_' or '@')
            {
                var escaped = current == '@'; var nameStart = escaped ? position : start;
                if (escaped && (position == source.Length || !char.IsAsciiLetter(source[position]) && source[position] != '_')) throw Invalid(start, "Expected an identifier after '@'.");
                while (position < source.Length && (char.IsAsciiLetterOrDigit(source[position]) || source[position] == '_')) position++;
                text = source[nameStart..position];
                kind = !escaped && text is "true" or "false" or "null" ? "literal" : !escaped && Keywords.Contains(text) || text == "_" ? "keyword" : "name";
            }
            else if (char.IsAsciiDigit(current) || current is '$' or '#')
            {
                var hex = current == '$'; var code = current == '#'; var real = false;
                if (hex || code) Digits(hex); else { position = start; Digits(false); }
                if (!hex && !code)
                {
                    if (position + 1 < source.Length && source[position] == '.' && char.IsAsciiDigit(source[position + 1])) { real = true; position++; Digits(false); }
                    if (position < source.Length && source[position] is 'e' or 'E') { real = true; position++; if (position < source.Length && source[position] is '+' or '-') position++; Digits(false); }
                    if (position < source.Length && source[position] is 'd' or 'D' or 'f' or 'F' or 'm' or 'M') position++;
                    else if (!real && position < source.Length && source[position] is 'l' or 'L' or 'u' or 'U')
                    { var first = char.ToLowerInvariant(source[position++]); if (position < source.Length && (first == 'u' && source[position] is 'l' or 'L' || first == 'l' && source[position] is 'u' or 'U')) position++; }
                }
                else if (hex && position < source.Length && source[position] is 'l' or 'L' or 'u' or 'U')
                { var first = char.ToLowerInvariant(source[position++]); if (position < source.Length && (first == 'u' && source[position] is 'l' or 'L' || first == 'l' && source[position] is 'u' or 'U')) position++; }
                if (position < source.Length && (char.IsAsciiLetter(source[position]) || source[position] == '_')) throw Invalid(start, "Invalid numeric literal suffix.");
                kind = "literal"; text = source[start..position];
            }
            else
            {
                kind = "symbol"; text = current.ToString();
                if (position < source.Length && (text is "<" or ">" or ":" && source[position] == '=' || text == "<" && source[position] == '>' || text == "?" && source[position] == '?')) text += source[position++];
                if (!";:.,()[]+-*/=<>?".Contains(current)) throw Invalid(start, "Unexpected expression character.");
            }
            result.Add(new(kind, text, start));
            if (result.Count > 8192) throw new DebugException("limit_exceeded", "The expression token limit was exceeded.");

            void Digits(bool hexadecimal)
            {
                var first = position;
                while (position < source.Length && (hexadecimal ? char.IsAsciiHexDigit(source[position]) : char.IsAsciiDigit(source[position])))
                {
                    position++;
                    if (position < source.Length && source[position] == '_')
                    { position++; if (position == source.Length || !(hexadecimal ? char.IsAsciiHexDigit(source[position]) : char.IsAsciiDigit(source[position]))) throw Invalid(start, "A digit separator must occur between digits."); }
                }
                if (position == first) throw Invalid(start, "A numeric literal requires digits.");
            }
        }
        result.Add(new("end", "<end>", source.Length)); return result.ToArray();
    }
    private static DebugException Invalid(int position, string message) => new("invalid_expression", message + $" Column {position + 1}.");
}
