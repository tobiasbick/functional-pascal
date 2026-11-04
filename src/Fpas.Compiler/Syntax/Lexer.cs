using System.Text;

namespace Fpas.Compiler.Syntax;

internal enum TokenKind { Identifier, Keyword, String, Integer, Real, Character, Symbol, End }

internal sealed record Token(TokenKind Kind, string Text, int Position, int Length, DocumentationComment? Documentation = null);

internal sealed class Lexer(SourceText source)
{
    private static readonly HashSet<string> Keywords = new(
        ("async await begin break case catch const continue discard distinct div do downto else " +
         "elsif end ensures enum extends false finally for function if implements in interface is mod new not null " +
         "of or out params private procedure program public raise range record ref repeat return " +
         "requires shl shr static subscribe then to true try type union unit unsubscribe until using var " +
         "when where while with and xor as uses class struct cast typeof unchecked").Split(' '), StringComparer.Ordinal);

    private int position;
    private int nextCancellationCheck;
    private char Current
    {
        get
        {
            if (position >= nextCancellationCheck)
            {
                source.CheckCancellation(); nextCancellationCheck = position + 1024;
            }
            return position < source.Text.Length ? source.Text[position] : '\0';
        }
    }
    private char Peek => position + 1 < source.Text.Length ? source.Text[position + 1] : '\0';

    internal IReadOnlyList<Token> ReadTokens()
    {
        var tokens = new List<Token>();
        var documentation = new List<DocumentationLine>();
        var newlines = 0;
        if (Current == '\uFEFF')
            position++;
        while (position < source.Text.Length)
        {
            if (char.IsWhiteSpace(Current))
            {
                if (Current is '\r' or '\n')
                {
                    if (++newlines > 1) documentation.Clear();
                    if (Current == '\r' && Peek == '\n') position++;
                }
                position++;
                continue;
            }
            if (Current == '/' && Peek == '/')
            {
                var start = position;
                var lineStart = start;
                while (lineStart > 0 && source.Text[lineStart - 1] is not ('\r' or '\n')) lineStart--;
                var standalone = source.Text.AsSpan(lineStart, start - lineStart).Trim().IsEmpty || lineStart == 0 && source.Text.AsSpan(0, start).Trim().SequenceEqual("\uFEFF");
                var textStart = position + 2;
                if (textStart < source.Text.Length && source.Text[textStart] == ' ') textStart++;
                while (position < source.Text.Length && Current is not '\r' and not '\n')
                    position++;
                if (standalone) documentation.Add(new(source.Text[textStart..position], textStart));
                else documentation.Clear();
                newlines = 0;
                continue;
            }
            if (Current == '#')
            {
                var start = position++;
                if (!char.IsAsciiDigit(Current)) throw source.Error("FP1004", "A character code requires decimal digits.", start);
                ReadDigits();
                tokens.Add(new(TokenKind.Character, source.Text[(start + 1)..position], start, position - start));
            }
            else if (Current == '\'')
                tokens.Add(ReadString());
            else if (char.IsAsciiLetter(Current) || Current is '_' or '@')
                tokens.Add(ReadIdentifier());
            else if (char.IsAsciiDigit(Current) || Current == '$')
                tokens.Add(ReadNumber());
            else
            {
                var start = position;
                var symbol = Current.ToString();
                if ((Current is ':' or '<' or '>') && Peek == '=' || Current == '<' && Peek == '>' || Current == '?' && Peek == '?' || Current == '=' && Peek == '>' || Current == '.' && Peek == '.')
                {
                    symbol += Peek;
                    position++;
                }
                else if (!";:.,()[]+-*/=<>?".Contains(Current))
                    throw source.Error("FP1001", $"Unexpected character '{Current}'.", position);
                position++;
                tokens.Add(new(TokenKind.Symbol, symbol, start, position - start));
            }
            if (documentation.Count != 0) tokens[^1] = tokens[^1] with { Documentation = new(documentation.ToArray()) };
            documentation.Clear(); newlines = 0;
        }
        tokens.Add(new(TokenKind.End, "<end of input>", position, 0));
        return tokens;
    }

    private Token ReadIdentifier()
    {
        var start = position;
        var escaped = Current == '@';
        if (escaped)
            position++;
        var nameStart = position;
        if (!char.IsAsciiLetter(Current) && Current != '_')
            throw source.Error("FP1001", "Expected an identifier after '@'.", start);
        while (char.IsAsciiLetterOrDigit(Current) || Current == '_')
            position++;
        var name = source.Text[nameStart..position];
        var kind = !escaped && Keywords.Contains(name) ? TokenKind.Keyword : TokenKind.Identifier;
        if (name == "_")
            kind = TokenKind.Symbol;
        return new(kind, name, start, position - start);
    }

    private Token ReadString()
    {
        var start = position++;
        var value = new StringBuilder();
        while (position < source.Text.Length)
        {
            if (Current is '\r' or '\n')
                break;
            if (Current == '\'')
            {
                position++;
                if (Current != '\'')
                    return new(TokenKind.String, value.ToString(), start, position - start);
            }
            value.Append(Current);
            position++;
        }
        throw source.Error("FP1002", "Unterminated string literal; strings cannot span source lines.", start, position - start);
    }

    private Token ReadNumber()
    {
        var start = position;
        var hex = Current == '$';
        if (hex) position++;
        if (hex) ReadHexDigits(); else ReadDigits();
        var real = false;
        if (!hex && Current == '.' && char.IsAsciiDigit(Peek))
        {
            real = true;
            position++;
            ReadDigits();
        }
        if (!hex && Current is 'e' or 'E')
        {
            real = true;
            position++;
            if (Current is '+' or '-')
                position++;
            if (!char.IsAsciiDigit(Current))
                throw source.Error("FP1004", "An exponent requires decimal digits.", start, position - start);
            ReadDigits();
        }
        if (!hex && Current is 'd' or 'D' or 'f' or 'F' or 'm' or 'M')
        {
            real = true;
            position++;
        }
        else if (!real && Current is 'l' or 'L' or 'u' or 'U')
        {
            var first = char.ToLowerInvariant(Current); position++;
            if (first == 'u' && Current is 'l' or 'L' || first == 'l' && Current is 'u' or 'U') position++;
        }
        if (char.IsAsciiLetter(Current) || Current == '_')
            throw source.Error("FP1004", "Invalid numeric literal or suffix.", start, position - start);
        return new(real ? TokenKind.Real : TokenKind.Integer, source.Text[start..position], start, position - start);
    }

    private void ReadHexDigits()
    {
        if (!char.IsAsciiHexDigit(Current)) throw source.Error("FP1004", "A hexadecimal literal requires digits.", position);
        while (char.IsAsciiHexDigit(Current))
        {
            position++;
            if (Current != '_') continue;
            var separator = position++;
            if (!char.IsAsciiHexDigit(Current)) throw source.Error("FP1003", "A numeric separator must occur singly between digits.", separator);
        }
    }

    private void ReadDigits()
    {
        while (char.IsAsciiDigit(Current))
        {
            position++;
            if (Current != '_')
                continue;
            var separator = position++;
            if (!char.IsAsciiDigit(Current))
                throw source.Error("FP1003", "A numeric separator must occur singly between digits.", separator);
        }
    }
}
