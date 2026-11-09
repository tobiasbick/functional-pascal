using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private int editorInvocation = -1;

    private void EditorExpression(ExpressionSyntax expression, bool check)
    {
        if (source.Editor is not { Context: "expression" or "member" } probe) return;
        if (expression is NameExpressionSyntax name && probe.Matches(name.Name))
        {
            probe.Reached = true;
            EditorLexical(probe);
            foreach (var (key, group) in recordContext?.UserMethods ?? []) AddRoutines(key, group);
            foreach (var (key, group) in routines) AddRoutines(key, group);
            foreach (var global in unit.Globals.Values) AddGlobal(probe, global);
            foreach (var alias in unit.Syntax.Imports)
                probe.Add(new(alias.Alias.Text, "namespace", alias.Target.Text));

            void AddRoutines(string key, IReadOnlyList<RoutineSymbol> group)
            {
                if (!HasValue(key))
                    foreach (var method in SourceMethods(group, null, name.Position, ImplicitReceiver(group, name.Position)))
                        AddMethod(probe, method);
            }
        }
        else if (expression is MemberSyntax member && probe.Matches(member.Name))
        {
            var target = StaticTarget(member.Receiver, allowOpenGeneric: true);
            if (target is NamespaceImport space)
            {
                probe.Reached = true;
                foreach (var candidate in references.EditorPaths(space.Namespace + ".", "", source.CancellationToken)) probe.Add(candidate);
                return;
            }
            if (target is SourceImport imported)
            {
                probe.Reached = true;
                foreach (var global in imported.Unit.Globals.Values.Where(global => global.Syntax.IsPublic || imported.Unit == unit)) AddGlobal(probe, global);
                foreach (var group in imported.Unit.Routines.Values)
                    foreach (var method in SourceMethods(group.Where(method => method.IsPublic || imported.Unit == unit).ToArray(), null, member.Position, null))
                        AddMethod(probe, method);
                return;
            }
            var owner = target is null ? Receiver(member.Receiver, check).Type : ImportedValueType(target, member.Position, allowOpen: true);
            if (owner == TypeSymbol.Unsupported) return;
            probe.Reached = true;
            if (owner.Kind == TypeKind.Array) probe.Add(new("Length", "property", "Length: int"));
            if (target is not null && references.UnionInfo(owner) is { } union)
                foreach (var item in union.Cases) AddMethod(probe, item.Factory, item.Name);
            foreach (var key in references.EditorMemberNames(owner).Distinct(StringComparer.Ordinal).Where(probe.Accepts))
            {
                source.CheckCancellation();
                try
                {
                    CheckRecordMember(owner, key, false, member.Name.Position);
                    if (references.Property(owner, key, target is not null, recordContext, source, member.Name.Position) is { Getter: { } getter } property)
                        probe.Add(new(key, "property", key + ": " + TypeFacts.Display(property.Type), EditorMetadata(getter)));
                    else if (references.Field(owner, key, target is not null) is { } field)
                        probe.Add(new(key, "field", key + ": " + TypeFacts.Display(field.Type), catalog.Analysis?.EditorField(field)));
                    else
                        foreach (var method in references.Methods(owner, key, target is not null, access: recordContext, includeOpen: true))
                            AddMethod(probe, method);
                }
                catch (CompilationFailure) { /* A candidate with inaccessible or unusable metadata is not offered. */ }
            }
        }
    }

    private void EditorLexical(EditorProbe probe)
    {
        foreach (var scope in scopes)
        {
            foreach (var local in scope.Values)
                probe.Add(new(local.Name, "variable", local.Name + ": " + TypeFacts.Display(local.Type)));
            if (ScopeRoutines(scope) is { } nested)
                foreach (var (name, group) in nested.Methods)
                    foreach (var method in NestedMethods(nested, group, null, false))
                        AddMethod(probe, method, name);
        }
        foreach (var parameter in parameters.Values)
            probe.Add(new(parameter.Name, "parameter", ParameterText(parameter, parameter.Type)));
        parent?.EditorLexical(probe);
    }

    private void AddMethod(EditorProbe probe, ImportedMethod method, string? name = null)
    {
        name ??= method.Name;
        if (!probe.Accepts(name)) return;
        probe.Add(new(name, "routine", MethodText(method, name), EditorMetadata(method), EditorDeclaration(method)));
    }

    private SourceSpan? EditorDeclaration(ImportedMethod method)
    {
        var symbol = method.SourceMethod;
        if (symbol?.Declaration?.Name is not { } token) return null;
        var owner = symbol.Owner?.Owner ?? symbol.InterfaceOwner?.Owner ??
            catalog.Units.FirstOrDefault(item => item.Routines.Values.Any(group => group.Contains(symbol))) ?? unit;
        return new SourceDocument(owner.Source.Path, owner.Source.Text, source.CancellationToken).Span(token.Position, token.Length);
    }

    private void AddGlobal(EditorProbe probe, GlobalSymbol global)
    {
        var token = global.Syntax.Binding.Name;
        if (!probe.Accepts(token.Text)) return;
        probe.Add(new(token.Text, "variable", token.Text + ": " + TypeFacts.Display(global.Type), Declaration:
            new SourceDocument(global.Owner.Source.Path, global.Owner.Source.Text, source.CancellationToken).Span(token.Position, token.Length)));
    }

    private MetadataIdentity? EditorMetadata(ImportedMethod method) => method.SourceMethod is null &&
        !method.DefinitionHandle.IsNil ? catalog.Analysis?.EditorMethod(method) : null;

    private static string ParameterText(ParameterSymbol parameter, TypeSymbol type)
        => (parameter.Mode.Length == 0 ? "" : parameter.Mode + " ") +
            (parameter.Name.Length == 0 ? "" : parameter.Name + ": ") + TypeFacts.Display(type) +
            (parameter.Default is null ? "" : " = default");

    private static string[] MethodParameters(ImportedMethod method)
        => method.Parameters.Select((type, index) => ParameterText(method.Parameter(index), type)).ToArray();

    private static string MethodText(ImportedMethod method, string name)
        => name + (method.MethodArguments is { Count: > 0 } variables ? "<" + string.Join(", ", variables.Select(TypeFacts.Display)) + ">" : "") +
            "(" + string.Join("; ", MethodParameters(method)) + ")" + (method.Result == TypeSymbol.Void ? "" : ": " + TypeFacts.Display(method.Result));

    private void EditorSignatures(IReadOnlyList<ImportedMethod> templates, IReadOnlyList<ExpressionSyntax> syntax, int position,
        IReadOnlyList<CallShape>? matches = null, CallShape? selected = null)
    {
        if (position != editorInvocation || source.Editor is not { Context: "signature" } probe) return;
        var items = new List<EditorCallSignature>();
        int? active = null;
        var slot = syntax.TakeWhile(argument => argument.SpanEnd < probe.Cursor).Count();
        var argumentName = slot < syntax.Count && syntax[slot] is NamedArgumentSyntax named ? named.Name.Text : probe.ArgumentName;
        if (probe.Incomplete) slot = Math.Min(slot, probe.Slot);
        foreach (var template in templates.Take(32))
        {
            var method = selected is not null && SameEditorMethod(selected.Method, template) ? selected.Method :
                matches?.FirstOrDefault(shape => SameEditorMethod(shape.Method, template))?.Method ?? template;
            var argument = new NameExpressionSyntax(new(TokenKind.Identifier, EditorAnalysis.Marker, position, 0));
            var prefix = syntax.Take(slot).ToList();
            prefix.Add(argumentName is { } name
                ? new NamedArgumentSyntax(new(TokenKind.Identifier, name, position, 0), argument)
                : argument);
            var mapped = Shape(method, prefix, false, allowMissing: true) ?? Shape(method, prefix, true, allowMissing: true);
            var parameter = mapped?.Slots[^1];
            var label = MethodText(method, method.Name);
            if (items.Any(item => item.Label == label)) continue;
            if (selected?.Method == method && !probe.Incomplete) active = items.Count;
            items.Add(new(label, MethodParameters(method), parameter, EditorMetadata(method), EditorDeclaration(method)));
        }
        probe.Signatures = items.AsReadOnly();
        probe.ActiveSignature = active ?? (items.Count == 0 ? null : items.FindIndex(item => item.ActiveParameter is not null) is var first && first >= 0 ? first : 0);
        probe.Incomplete |= templates.Count > 32;
    }

    private static bool SameEditorMethod(ImportedMethod candidate, ImportedMethod template)
        => ReferenceEquals(candidate, template) || candidate.Owner == template.Owner &&
            (template.SourceMethod is { } source ? ReferenceEquals(candidate.SourceMethod, source) :
                !template.DefinitionHandle.IsNil && candidate.DefinitionHandle == template.DefinitionHandle);
}
