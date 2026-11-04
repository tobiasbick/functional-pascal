using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private readonly Dictionary<BoundMethodGroup, List<(SourceText Source, ExpressionSyntax Syntax)>> methodGroups = new(ReferenceEqualityComparer.Instance);

    private static BoundExpression Unwrap(BoundExpression value) => value switch
    {
        BoundSequence sequence => Unwrap(sequence.Value), BoundConversion conversion => Unwrap(conversion.Operand), _ => value
    };

    private SymbolEntry External(Analysis.MetadataIdentity identity, string kind, string name)
    {
        var key = Key(identity);
        if (!metadataSymbols.TryGetValue(key, out var entry))
        {
            entry = Entry(new object(), kind, name); Export(identity, entry); entry.Metadata = identity;
            // Definitions retain original CLR identities; substitutions belong to expression facts.
            entry.Signature = kind == "type" ? identity.Type : identity.Type + "." + name +
                (identity.Kind == "method" ? "(" + string.Join(", ", identity.Parameters) + "): " + identity.Result : "");
        }
        return entry;
    }

    private SymbolEntry Method(ImportedMethod method)
    {
        if (method.SourceMethod is { } source)
            return Entry(source, "routine", source.Name);
        return External(MethodMetadata(method), "routine", method.Name);
    }

    private SymbolEntry Property(ImportedMethod getter, string? writtenName = null)
    {
        if (getter.SourceMethod is { } source && symbols.TryGetValue(source, out var own)) return own;
        var name = writtenName ?? (getter.Name.StartsWith("get_", StringComparison.Ordinal) || getter.Name.StartsWith("set_", StringComparison.Ordinal) ? getter.Name[4..] : getter.Name);
        return External(Metadata(getter.Owner, "property", name), "property", name);
    }

    private SymbolEntry? Resolved(BoundExpression value) => Unwrap(value) switch
    {
        BoundLocal local => ValueSymbol(local.Local, "local", local.Local.Name, local.Local.Type),
        BoundParameter parameter => ValueSymbol(parameter.Parameter, "parameter", parameter.Parameter.Name, parameter.Parameter.Type),
        BoundCapture capture => capture.Capture.Original switch
        {
            LocalSymbol local => ValueSymbol(local, "local", local.Name, local.Type), ParameterSymbol parameter => ValueSymbol(parameter, "parameter", parameter.Name, parameter.Type),
            _ => throw new InvalidOperationException("Unrecognized capture identity.")
        },
        BoundRecordField field => Entry(field.Field, "property", field.Field.Name),
        BoundField field => External(Metadata(field.Field.Owner, "field", field.Field.Name), "field", field.Field.Name),
        BoundProperty property when (property.Property.Getter ?? property.Property.Setter) is { } accessor => Property(accessor),
        BoundMethodDelegate method => Method(method.Method),
        BoundCall call => Method(call.Method),
        BoundArrayLength => External(Metadata(references.Symbol(references.Required("System.Array")), "property", "Length"), "property", "Length"),
        _ => null
    };

    private SymbolEntry ValueSymbol(object identity, string kind, string name, TypeSymbol type)
    {
        var entry = Entry(identity, kind, name);
        entry.Type ??= Type(type);
        if (entry.Declaration is null) entry.Signature = name + ": " + TypeFacts.Display(type);
        return entry;
    }

    private static Token? NameToken(ExpressionSyntax syntax) => syntax switch
    {
        NameExpressionSyntax name => name.Name, MemberSyntax member => member.Name,
        GenericNameSyntax generic => NameToken(generic.Name), ParenthesizedSyntax parenthesized => NameToken(parenthesized.Expression),
        _ => null
    };

    internal void Expression(SourceText source, ExpressionSyntax syntax, BoundExpression bound, string role = "read")
    {
        token.ThrowIfCancellationRequested();
        if (!SemanticPolicy.Expressions.ContainsKey(bound.GetType()) || !SemanticPolicy.Syntax.Contains(syntax.GetType()))
            throw new InvalidOperationException("Missing semantic indexing policy for " + syntax.GetType().Name + "/" + bound.GetType().Name + ".");
        var value = Unwrap(bound);
        var written = syntax is ArgumentModeSyntax argument ? argument.Value : syntax;
        if (value is BoundMethodGroup group)
        {
            if (!methodGroups.TryGetValue(group, out var selections)) methodGroups.Add(group, selections = []);
            selections.Add((source, written)); return;
        }
        var selectedType = value is BoundMethodDelegate method ? Callable(method.Method) : Type(bound.Type);
        if (syntax is MemberSyntax && value is BoundCall memberCall) RefineReceiver(source, syntax, memberCall.Method);
        var declared = value switch
        {
            BoundLocal local => Type(local.Local.Type), BoundParameter parameter => Type(parameter.Parameter.Type),
            BoundCapture capture => Type(capture.Capture.Type), _ => null
        };
        if (syntax.SpanEnd > syntax.SpanStart) Typed(source, syntax.SpanStart, syntax.SpanEnd - syntax.SpanStart, selectedType, declared);
        if (syntax is CallSyntax call && value is BoundCall invocation)
        {
            // Delegate invocation already binds its explicit receiver; Invoke is synthesized.
            if (invocation.Method.Name == "Invoke" && invocation.Method.SourceMethod is null && references.DelegateSignature(invocation.Method.Owner) is not null) return;
            UseMethod(source, call.Target, invocation.Method); return;
        }
        if (written is NameExpressionSyntax or MemberSyntax or GenericNameSyntax && NameToken(written) is { } name && Resolved(bound) is { } symbol)
        {
            Occurrence(source, name, symbol, role);
            Typed(source, name.Position, name.Length, selectedType, declared);
        }
    }

    private void UseMethod(SourceText source, ExpressionSyntax syntax, ImportedMethod method)
    {
        if (NameToken(syntax) is not { } name) return;
        Occurrence(source, name, Method(method), "read");
        var type = Callable(method);
        RefineReceiver(source, syntax, method);
        Typed(source, name.Position, name.Length, type);
        if (syntax.SpanEnd > syntax.SpanStart) Typed(source, syntax.SpanStart, syntax.SpanEnd - syntax.SpanStart, type);
    }

    private void RefineReceiver(SourceText source, ExpressionSyntax syntax, ImportedMethod method)
    {
        if (syntax is GenericNameSyntax generic) syntax = generic.Name;
        if (syntax is not MemberSyntax member || NameToken(member.Receiver) is not { } name) return;
        if (!expressions.TryGetValue((source.Path, name.Position, name.Length), out var previous)) return;
        var metadata = types[previous.Type].Metadata;
        if (metadata is null || metadata.Kind != "type" || metadata.Type != TypeName(method.Owner)) return;
        var type = Type(method.Owner);
        Typed(source, name.Position, name.Length, type);
        if (member.Receiver.SpanEnd > member.Receiver.SpanStart) Typed(source, member.Receiver.SpanStart, member.Receiver.SpanEnd - member.Receiver.SpanStart, type);
    }

    internal void Retarget(BoundMethodGroup group, ImportedMethod method)
    {
        if (!methodGroups.TryGetValue(group, out var selections)) return;
        foreach (var (source, syntax) in selections) UseMethod(source, syntax, method);
    }

    internal void ResolvedMethod(SourceText source, Token name, ImportedMethod method)
    {
        Occurrence(source, name, Method(method), "read");
        Typed(source, name.Position, name.Length, Callable(method));
    }

    internal void NamedParameter(SourceText source, Token name, ImportedMethod method, int slot)
    {
        SymbolEntry symbol;
        if (method.SourceMethod is { } sourceMethod) symbol = Entry(sourceMethod.Parameters[slot], "parameter", sourceMethod.Parameters[slot].Name);
        else
        {
            var identity = MethodMetadata(method) with { Kind = "parameter", Name = method.Name + "#" + slot };
            symbol = External(identity, "parameter", method.Parameter(slot).Name);
        }
        Occurrence(source, name, symbol, "argument"); Typed(source, name.Position, name.Length, Type(method.Parameters[slot]));
    }

    internal void NamedProperty(SourceText source, Token name, ImportedMethod getter, string role)
    {
        var symbol = Property(getter, name.Text); Occurrence(source, name, symbol, role);
        Typed(source, name.Position, name.Length, Type(getter.Result));
    }

    internal void Event(SourceText source, Token name, ImportedMethod accessor, TypeSymbol type)
    {
        var symbol = External(Metadata(accessor.Owner, "event", name.Text), "event", name.Text);
        Occurrence(source, name, symbol, "write"); Typed(source, name.Position, name.Length, Type(type));
    }

    internal void AttributeMember(SourceText source, Token name, TypeSymbol owner, TypeSymbol type, bool field)
    {
        var kind = field ? "field" : "property";
        var symbol = External(Metadata(owner, kind, name.Text), kind, name.Text);
        Occurrence(source, name, symbol, "write"); Typed(source, name.Position, name.Length, Type(type));
    }

    internal void EnumMemberUse(SourceText source, int position, ScalarMember member)
    {
        if (TokenAt(source, position) is { } name) Occurrence(source, name, Entry(member, "enum_member", member.Name.Text), "read");
    }
}
