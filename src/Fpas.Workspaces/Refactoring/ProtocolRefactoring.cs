// [Documentation](../../../docs/pascal/tools/refactoring.md#implement-a-protocol)
using System.Text;
using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> ImplementProtocol(Draft draft, RefactorProtocol? protocol, SourceSelection selection)
    {
        if (protocol is not { IsValid: true })
            throw Refuse("Supply a protocol type and at most one implementation per required member.");
        var definition = draft.Before.Query.Execute("definition", selection, cancellationToken: draft.Token).Definition;
        if (definition is not { Origin: "source", Symbol: { Kind: "record", Declaration: { } declaration } })
            throw Refuse("Select an editable record declaration or reference.");
        var source = draft.Source(declaration.Path);
        var syntax = draft.Syntax(source);
        if (syntax.Record(declaration.Offset) is not { } record)
            throw Refuse("Select an editable record declaration or reference.");

        var type = protocol.Type.Trim();
        ProtocolRequirements? requirements = null;
        foreach (var (owner, snapshot) in draft.Owners(source.Path))
        {
            var project = draft.Analysis.Context.Projects.Single(item => Paths.Equals(item.Path, owner));
            var actual = Inspect(project.Request, source.Path, declaration.Offset, type);
            if (requirements is not null && (requirements.AlreadyImplemented != actual.AlreadyImplemented ||
                    !requirements.Members.SequenceEqual(actual.Members) || !requirements.Imports.SequenceEqual(actual.Imports)))
                throw Refuse("Linked owners require different implementations for this protocol.");
            requirements = actual;
            var symbol = snapshot.Symbols.Single(item => item.Kind == "record" && item.Declaration is { } span &&
                Paths.Equals(span.Path, declaration.Path) && span.Offset == declaration.Offset);
            draft.Targets.Add(new(owner, symbol.Kind, symbol.Name, symbol.Declaration!, RenameBindings.IsPublic(snapshot, symbol)));
        }
        if (requirements is null) throw Refuse("The record's source has no checked project owner.");
        var supplied = protocol.Members.ToDictionary(member => member.Key, StringComparer.Ordinal);
        if (supplied.Keys.Except(requirements.Members.Select(member => member.Key), StringComparer.Ordinal).FirstOrDefault() is { } unknown)
            throw Refuse($"'{unknown}' does not identify a missing member.");
        if (requirements.AlreadyImplemented) return ["The record already implements " + type + "."];
        var missing = requirements.Members.Where(member => !supplied.ContainsKey(member.Key)).ToArray();
        if (missing.Length != 0)
            throw Refuse("Supply an explicit implementation for each missing member. " +
                string.Join(" ", missing.Select(member => member.Key + " = " + member.Header.ReplaceLineEndings(" "))));

        draft.ImplementedRecord = declaration;
        draft.CheckOverloads = true;
        draft.Edit(source, record.HeaderEnd, record.HeaderEnd, (record.HasInterfaces ? ", " : " implements ") + type);
        var newline = LineEnding(source.Text, record.BodyEnd);
        if (requirements.Imports.Count != 0)
        {
            var imports = string.Join(newline, requirements.Imports.Select(import => "uses " + import + ";"));
            if (syntax.Imports() is { } existing)
            {
                var end = existing.End;
                var lineEnd = source.Text.IndexOfAny(['\r', '\n'], end);
                if (lineEnd >= 0 && source.Text.AsSpan(end, lineEnd - end).TrimStart().StartsWith("//", StringComparison.Ordinal))
                    end = lineEnd;
                draft.Edit(source, end, end, newline + imports);
            }
            else
            {
                // Insert before the first declaration's trivia, without moving a trailing header comment.
                var end = syntax.ContainerHeaderEnd;
                var lineEnd = source.Text.IndexOfAny(['\r', '\n'], end);
                var tail = lineEnd >= 0 ? source.Text[end..lineEnd].TrimStart() : "";
                if (tail.StartsWith("//", StringComparison.Ordinal) || string.IsNullOrWhiteSpace(tail))
                    end = lineEnd < 0 ? end : lineEnd;
                draft.Edit(source, end, end, newline + newline + imports +
                    (end == lineEnd ? "" : newline));
            }
        }
        if (requirements.Members.Count == 0)
            return ["Compatible record members and inherited implementations are reused."];

        var lineStart = source.Text.LastIndexOfAny(['\r', '\n'], record.BodyEnd - 1) + 1;
        var leading = source.Text[lineStart..record.BodyEnd];
        var ownLine = leading.All(character => character is ' ' or '\t');
        var indent = (ownLine ? leading : "") + "  ";
        var insertion = ownLine ? lineStart : record.BodyEnd;
        var text = new StringBuilder();
        var previousEnd = lineStart;
        if (previousEnd > 0 && source.Text[previousEnd - 1] == '\n') previousEnd--;
        if (previousEnd > 0 && source.Text[previousEnd - 1] == '\r') previousEnd--;
        var before = source.Text.AsSpan(0, previousEnd);
        var previousLine = before[(before.LastIndexOfAny('\r', '\n') + 1)..];
        if (!ownLine || previousLine.TrimStart().StartsWith("//", StringComparison.Ordinal))
            text.Append(newline);
        var firstMember = true;
        foreach (var member in requirements.Members.OrderBy(member => member.Arguments is not null))
        {
            var implementation = supplied[member.Key];
            var value = implementation.Value.Trim();
            string? forward = null;
            if (implementation.Forward)
            {
                if (member.Arguments is null)
                    throw Refuse($"Getter-only property '{member.Key}' requires an explicit initializer.");
                if (value == member.Name) throw Refuse($"'{member.Key}' cannot forward to itself.");
                try { forward = IdentifierFacts.Spell(value); }
                catch (ArgumentException error) { throw Refuse(error.Message); }
                var call = "self." + forward + member.Arguments;
                value = (member.ReturnsValue ? "return " : "") + call + ";";
            }
            try
            {
                if (!firstMember && member.Arguments is not null) text.Append(newline);
                firstMember = false;
                var written = member.Complete(value, indent, newline, draft.Token);
                if (forward is not null)
                {
                    var anchor = written.IndexOf("self." + forward, StringComparison.Ordinal) + "self.".Length;
                    draft.ForwardedMethods.Add((implementation.Value.Trim(), insertion, text.Length + anchor));
                }
                text.Append(written).Append(newline);
            }
            catch (ArgumentException error) { throw Refuse(error.Message); }
        }
        draft.Edit(source, insertion, insertion, text.ToString());
        return ["Compatible record members and inherited implementations are reused. Missing members use the supplied behavior."];

        ProtocolRequirements Inspect(AnalysisRequest input, string path, int offset, string requested)
        {
            try { return ProtocolImplementation.Inspect(input, path, offset, requested, draft.Token); }
            catch (ArgumentException error) { throw Refuse(error.Message); }
        }
    }
}
