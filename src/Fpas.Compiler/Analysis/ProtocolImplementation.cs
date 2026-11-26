// [Documentation](../../../docs/pascal/tools/refactoring.md#implement-a-protocol)
using System.Globalization;
using System.Text;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

/// <summary>Describes a missing protocol member in the destination record's scope.</summary>
/// <param name="Key">The member selector, including an overload number when needed.</param>
/// <param name="Name">The unescaped record member name.</param>
/// <param name="Header">The complete method header or property declaration up to the initializer.</param>
/// <param name="Arguments">Type and value arguments for forwarding, or null for a property.</param>
/// <param name="ReturnsValue">Whether a method returns a value.</param>
public sealed record ProtocolMember(string Key, string Name, string Header, string? Arguments, bool ReturnsValue)
{
    /// <summary>Completes the generated member with explicit FPAS statements or a property initializer.</summary>
    /// <param name="body">Method statements or a property expression.</param>
    /// <param name="indent">Indentation of a record member.</param>
    /// <param name="newline">Line ending used in the destination source.</param>
    /// <param name="token">Cancels the syntax check.</param>
    /// <returns>One self-contained member declaration.</returns>
    public string Complete(string body, string indent, string newline, CancellationToken token)
    {
        body = body.Replace("\r\n", "\n", StringComparison.Ordinal).Replace('\r', '\n');
        const string prefix = "unit Fragment; type Adapter = record\n";
        const string suffix = "\nend record; end unit;";
        var kind = ReturnsValue ? "function" : "procedure";
        var closing = "\nend " + kind + ";";
        var head = Arguments is null ? Header + " := " : Header + "\nbegin\n";
        var member = head + body + (Arguments is null ? ";" : closing);
        try
        {
            var source = new SourceText("<refactoring>", prefix + member + suffix, token);
            var tokens = new Lexer(source).ReadTokens();
            var parsed = new Parser(source, tokens).ParseCompilationUnit();
            if (parsed.Records is not [var record])
                throw new ArgumentException($"Supply one complete implementation for '{Key}'.");
            var valid = Arguments is null
                ? record is { Methods.Count: 0, Fields: [{ Default: { } value }] } &&
                    value.SpanEnd == prefix.Length + member.Length - 1
                : record is { Fields.Count: 0, Methods: [var method] } &&
                    parsed.Declarations[method.Name.Position].End == prefix.Length + member.Length;
            if (valid && parsed.Declarations[record.Name.Position].BodyEnd == prefix.Length + member.Length + 1)
                return Indent(member, head.Length, body.Length, prefix.Length, indent, newline, tokens);
        }
        catch (CompilationFailure failure)
        {
            throw new ArgumentException($"Invalid implementation for '{Key}': {failure.Diagnostic.Message}");
        }
        throw new ArgumentException($"Supply one complete implementation for '{Key}'.");
    }

    private static string Indent(string declaration, int bodyStart, int bodyLength, int offset, string indent, string newline, IReadOnlyList<Token> tokens)
    {
        var literals = new Queue<Token>(tokens.Where(token => token.Kind == TokenKind.String));
        var result = new StringBuilder();
        var start = 0;
        foreach (var line in declaration.Split('\n'))
        {
            while (literals.TryPeek(out var previous) && previous.Position + previous.Length <= offset + start)
                literals.Dequeue();
            var insideString = literals.TryPeek(out var literal) && literal.Position < offset + start;
            if (start > 0) result.Append(newline);
            if (line.Length != 0 && !insideString)
            {
                result.Append(indent);
                if (start >= bodyStart && start < bodyStart + bodyLength) result.Append("  ");
            }
            result.Append(line);
            start += line.Length + 1;
        }
        return result.ToString();
    }
}

/// <summary>The compiler's requirements for adding one protocol to a record.</summary>
/// <param name="AlreadyImplemented">Whether the record already provides the protocol through its declared interfaces.</param>
/// <param name="Members">The members that need an explicit implementation.</param>
/// <param name="Imports">Additional written uses entries needed by the generated declarations.</param>
public sealed record ProtocolRequirements(bool AlreadyImplemented, IReadOnlyList<ProtocolMember> Members, IReadOnlyList<string> Imports);

/// <summary>Inspects protocol slots using the compiler's record conformance rules.</summary>
public static class ProtocolImplementation
{
    /// <summary>Resolves the protocol in the scope of an existing, checked source record.</summary>
    /// <param name="request">The owning project's captured analysis inputs.</param>
    /// <param name="path">The record's source path.</param>
    /// <param name="offset">The offset of the record's name.</param>
    /// <param name="type">FPAS protocol or CLR interface type syntax.</param>
    /// <param name="token">Cancels compiler work.</param>
    /// <returns>The resolved requirements or an argument error for an unsupported request.</returns>
    public static ProtocolRequirements Inspect(AnalysisRequest request, string path, int offset, string type, CancellationToken token)
    {
        try
        {
            using var compilation = PreparedCompilation.Create(request, token);
            var record = compilation.Catalog.Units.Where(unit => unit.Source.Path == path).SelectMany(unit => unit.Records.Values)
                .SingleOrDefault(item => item.Syntax.Name.Position == offset)
                ?? throw new ArgumentException("Select an editable FPAS record.");
            var references = compilation.References;
            var fragment = new SourceText("<protocol>", type, token);
            var syntax = new Parser(fragment, new Lexer(fragment).ReadTokens()).ParseStandaloneType();
            if (syntax.Position != 0 || syntax.SpanEnd != type.Length)
                throw new ArgumentException("Supply a complete protocol type without surrounding comments.");
            var scope = record.GenericParameters.ToDictionary(parameter => parameter.Name, StringComparer.Ordinal);
            var contract = compilation.Catalog.ResolveType(syntax, record.Owner, references, scope);
            if (!contract.IsInterface || contract.IsNullable)
                throw new ArgumentException("Supply a non-nullable protocol or CLR interface type visible in the record's unit.");
            if (record.Interfaces.Any(item => references.Derives(item, contract)))
                return new(true, [], []);

            var owner = record.Owner;
            var imports = new Dictionary<string, string>(StringComparer.Ordinal);
            var used = owner.Syntax.Imports.Select(item => item.Alias.Text).Concat(owner.Records.Keys)
                .Concat(owner.Interfaces.Keys).Concat(owner.Variants.Keys).Concat(owner.Scalars.Keys).Concat(owner.Aliases.Keys)
                .Concat(scope.Keys).ToHashSet(StringComparer.Ordinal);
            var missing = references.InterfaceRequirements([.. record.Interfaces, contract], owner.Source, offset)
                .Where(item => SourceCatalog.MatchInterfaceMember(record, references, item.Method, item.Required, allowMissing: true).Missing)
                .Select(item => item.Method)
                .GroupBy(method => (method.Name, references.InterfaceGetter(method), Arity: method.MethodArguments?.Count ?? 0,
                    Parameters: new TypeList(method.DelegateParameters.Select(parameter =>
                        references.SourceSignature(parameter, record.GenericParameters).Erased))))
                .Select(MergeSlots);
            var members = missing.Select(Render).OrderBy(member => member.Header, StringComparer.Ordinal).GroupBy(member => member.Name)
                .SelectMany(group => group.Count() == 1 ? group : group.Select((member, index) =>
                    member with { Key = member.Name + "#" + (index + 1).ToString(CultureInfo.InvariantCulture) })).ToArray();
            return new(false, members, imports.Select(pair => FpasIdentifiers.SpellQualified(pair.Key) + " as " + IdentifierFacts.Spell(pair.Value)).ToArray());

            ProtocolMember Render(ImportedMethod method)
            {
                var generic = references.InterfaceGenericConstraints(method);
                var names = method.SourceMethod is not null
                    ? generic.Select(parameter => IdentifierFacts.Spell(parameter.Name)).ToArray()
                    : generic.Select((parameter, index) => IdentifierFacts.IsValid(parameter.Name) ? IdentifierFacts.Spell(parameter.Name) :
                        "Type" + (index + 1).ToString(CultureInfo.InvariantCulture)).ToArray();
                string Spell(TypeSymbol value) => Type(value, method, names, Qualify, owner);
                if (references.InterfaceGetter(method))
                {
                    var property = method.Name["get_".Length..];
                    return new(property, property, "public " + IdentifierFacts.Spell(property) + ": " + Spell(method.Result), null, true);
                }
                var parameters = Enumerable.Range(0, method.Parameters.Count).Select(method.Parameter).ToArray();
                var usedNames = names.Select(name => name.TrimStart('@')).ToHashSet(StringComparer.Ordinal);
                var labels = parameters.Select((parameter, index) => Unique(usedNames,
                    IdentifierFacts.IsValid(parameter.Name) && parameter.Name != "self" ? parameter.Name :
                        "Argument" + (index + 1).ToString(CultureInfo.InvariantCulture))).Select(IdentifierFacts.Spell).ToArray();
                var arguments = FpasIdentifiers.ApplyTypeArguments("", names);
                var returnsValue = method.Result.Kind != TypeKind.Void;
                var header = "public " + (returnsValue ? "function " : "procedure ") + IdentifierFacts.Spell(method.Name) + arguments +
                    "(" + string.Join("; ", parameters.Select((parameter, index) =>
                        Mode(parameter) + labels[index] + ": " + Spell(parameter.Type))) + ")" +
                    (returnsValue ? ": " + Spell(method.Result) : "") + ";";
                for (var index = 0; index < generic.Count; index++)
                {
                    var parameter = generic[index];
                    var constraints = new List<string>();
                    if (parameter.Primary is { } primary) constraints.Add(primary);
                    constraints.AddRange(parameter.Bounds.Select(Spell));
                    if (parameter.Constructor) constraints.Add("new()");
                    if (constraints.Count != 0)
                        header += "\nwhere " + names[index] + ": " + string.Join(", ", constraints) + ";";
                }
                return new(method.Name, method.Name, header,
                    arguments + "(" + string.Join(", ", parameters.Select((parameter, index) => Mode(parameter) + labels[index])) + ")", returnsValue);
            }

            string Qualify(string name)
            {
                var existing = owner.Syntax.Imports.Where(item => name == item.Target.Text || name.StartsWith(item.Target.Text + ".", StringComparison.Ordinal))
                    .OrderByDescending(item => item.Target.Text.Length).FirstOrDefault();
                if (existing is not null)
                    return IdentifierFacts.Spell(existing.Alias.Text) + (existing.Target.Text.Length == name.Length ? "" :
                        "." + FpasIdentifiers.SpellQualified(name[(existing.Target.Text.Length + 1)..]));
                if (!imports.TryGetValue(name, out var alias))
                    imports.Add(name, alias = Unique(used, name[(name.LastIndexOf('.') + 1)..]));
                return IdentifierFacts.Spell(alias);
            }
        }
        catch (CompilationFailure failure)
        {
            throw new ArgumentException(failure.Diagnostic.Message);
        }
    }

    private static ImportedMethod MergeSlots(IEnumerable<ImportedMethod> slots)
    {
        var methods = slots.ToArray();
        var first = methods[0];
        if (methods.Length == 1) return first;

        // The implementation must accept all permitted inputs and provide the strongest result promise.
        var parameters = Enumerable.Range(0, first.Parameters.Count).Select(index =>
        {
            var parameter = first.Parameter(index);
            var properties = methods.Select(method => method.Parameters[index]);
            var nullable = parameter.Mode switch
            {
                "out" => properties.All(type => type.IsNullable),
                "ref" => parameter.Type.IsNullable,
                _ => properties.Any(type => type.IsNullable)
            };
            return parameter with { Type = WithNullability(parameter.Type, nullable) };
        }).ToArray();
        return first with
        {
            Parameters = parameters.Select(parameter => parameter.Type).ToArray(),
            ParameterInfo = parameters,
            Result = WithNullability(first.Result, methods.All(method => method.Result.IsNullable))
        };

        static TypeSymbol WithNullability(TypeSymbol type, bool nullable)
            => type.IsValue ? type : type with { IsNullable = nullable, NullValidated = false };
    }

    private static string Type(TypeSymbol type, ImportedMethod method, string[] names, Func<string, string> qualify, SourceUnit owner)
    {
        string Spell(TypeSymbol argument) => Type(argument, method, names, qualify, owner);
        string Local(SourceUnit unit, string name) => (unit == owner ? "" : qualify(unit.Syntax.Name.Text) + ".") + FpasIdentifiers.SpellQualified(name);
        var nullable = type.IsNullable ? "?" : "";
        if (type.Kind == TypeKind.Array)
        {
            var rank = type.Rank == 1 ? "" : "[" + new string(',', type.Rank - 1) + "]";
            var array = "array" + rank + " of " + Spell(type.Element!);
            return type.IsNullable ? "(" + array + ")?" : array;
        }
        if (type.Kind == TypeKind.MethodParameter) return names[type.ParameterIndex] + nullable;
        if (type.Parameter is { } parameter)
            return (method.SourceMethod?.GenericParameters.Contains(parameter) == true
                ? names[parameter.Index] : IdentifierFacts.Spell(parameter.Name)) + nullable;
        if (type.Definition is { } definition)
            return UsageNames.Named(definition, type.Arguments ?? TypeList.Empty, Spell, qualify, type.IsNullable) + nullable;
        var arguments = type.Arguments?.Select(Spell).ToArray() ?? [];
        if (type.Record is { } record)
            return (record.VariantCase is { } item ? FpasIdentifiers.ApplyTypeArguments(Local(record.Owner, item.Owner.Name), arguments, groupBeforeSuffix: true) + "." +
                IdentifierFacts.Spell(record.Name) : Applied(record.Owner, record.Name)) + nullable;
        if (type.Interface is { } contract) return Applied(contract.Owner, contract.Name) + nullable;
        if (type.Variant is { } variant) return Applied(variant.Owner, variant.Name) + nullable;
        if (type.Scalar is { } scalar) return Local(scalar.Owner, scalar.Name) + nullable;
        return type.ToString();

        string Applied(SourceUnit unit, string name) => FpasIdentifiers.ApplyTypeArguments(Local(unit, name), arguments, type.IsNullable);
    }

    private static string Unique(HashSet<string> used, string stem)
    {
        var name = stem;
        for (var suffix = 2; !used.Add(name); suffix++) name = stem + suffix.ToString(CultureInfo.InvariantCulture);
        return name;
    }

    private static string Mode(ParameterSymbol parameter) => parameter.ByRef ? parameter.Mode + " " : "";
}
