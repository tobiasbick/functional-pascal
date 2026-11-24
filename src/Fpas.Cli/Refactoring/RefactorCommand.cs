// [Documentation](../../../docs/pascal/tools/refactoring.md)
using Fpas.Cli.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Refactoring;
using System.CommandLine;
using System.Globalization;
using System.Text.Json;
using System.Text.RegularExpressions;

namespace Fpas.Cli.Refactoring;

internal sealed class RefactorCommand
{
    private sealed class Operation(string name)
    {
        internal ContextCommandOptions Context { get; } = new();
        internal Option<string> ExpectedSnapshot { get; } = SnapshotGuard.CreateOption();
        internal Option<string> Symbol { get; } = SourceSymbolOptions.Create();
        internal bool AcceptsSymbol => name is "rename" or "add-parameter" or "remove-parameter" or "change-signature" or "implement-protocol" or "inline-variable";
        internal Option<string> Overlay { get; } = new("--overlay") { Required = true, Description = "JSON draft source texts to validate and plan." };
        internal Option<string> Plan { get; } = new("--plan") { Required = true, Description = "Explicit .json plan path; preparation creates it without overwriting, apply reads it." };
        internal Option<string> File { get; } = new("--file") { Required = true, Description = "Source file to refactor." };
        internal Option<int> Line { get; } = new("--line") { Required = true, Description = "One-based source line." };
        internal Option<int> Column { get; } = new("--column") { Required = true, Description = "One-based UTF-16 source column." };
        internal Option<int> EndLine { get; } = new("--end-line") { Required = true, Description = "One-based line at the exclusive end of the selected source range." };
        internal Option<int> EndColumn { get; } = new("--end-column") { Required = true, Description = "One-based UTF-16 column at the exclusive end of the selected source range." };
        internal Option<string> Project { get; } = new("--project") { Description = "Owning FPAS project for a linked source selection." };
        internal Option<string> NewName { get; } = new("--new-name") { Required = true, Description = "Unescaped simple ASCII identifier; keyword escaping is chosen automatically." };
        internal Option<string> Name { get; } = new("--name") { Required = true, Description = "Unescaped identifier for the new declaration." };
        internal Option<string> ParameterType { get; } = new("--type") { Required = name == "add-parameter", Description = "Explicit FPAS parameter type." };
        internal Option<string> Argument { get; } = new("--argument") { Required = true, Description = "FPAS value expression inserted at each direct call." };
        internal Option<string> Mode { get; } = new("--mode") { Description = "New parameter mode: value, in, ref or out." };
        internal Option<string> Order { get; } = new("--order")
        {
            Description = "All unescaped parameter names in the requested order, separated by commas. Select the routine or a direct call."
        };
        internal Option<string[]> CallArguments { get; } = new("--argument")
        {
            Description = "Replace one argument at [FILE:]LINE:COLUMN=EXPRESSION. Omit FILE for the routine's source. Repeat for each replacement.",
            AllowMultipleArgumentsPerToken = false,
            Arity = ArgumentArity.OneOrMore
        };
        internal Option<string> Protocol { get; } = new("--protocol")
        {
            Required = true, Description = "FPAS protocol or CLR interface type as written in the selected record's unit."
        };
        internal Option<string[]> Members { get; } = new("--member")
        {
            Description = "Supply MEMBER=STATEMENTS for a method or MEMBER=EXPRESSION for a getter-only property. Repeat for each member.",
            AllowMultipleArgumentsPerToken = false, Arity = ArgumentArity.OneOrMore
        };
        internal Option<string[]> Forward { get; } = new("--forward")
        {
            Description = "Supply MEMBER=METHOD to delegate to an existing instance method of the record. Repeat for each member.",
            AllowMultipleArgumentsPerToken = false, Arity = ArgumentArity.OneOrMore
        };
        internal Option<bool> ClosedContext { get; } = new("--closed-context")
        {
            Description = "Confirm the selected context includes every consumer of an edited public signature."
        };
        internal Option<string> Report { get; } = new("--report") { Description = "Write one JSON response to stdout." };
        internal Command Command { get; } = new(name, name == "apply" ? "Revalidate and explicitly apply a checked refactoring plan." : $"Prepare a checked {name} plan without changing sources.");
        internal bool Json(ParseResult result) => result.GetResult(Report)?.Tokens.Any(token => token.Value == "json") == true;
        internal void Bind(RefactorCommand owner)
        {
            Context.AddTo(Command, preparationOptions: name != "draft"); Report.AcceptValues("json"); Command.Options.Add(Plan); Command.Options.Add(Report);
            if (name != "apply") Command.Options.Add(ExpectedSnapshot);
            if (name == "draft") Command.Options.Add(Overlay);
            else if (name != "apply")
            {
                Command.Options.Add(File);
                Command.Options.Add(Project);
                if (name is "add-parameter" or "remove-parameter" or "change-signature") Command.Options.Add(ClosedContext);
                if (name != "organize-imports")
                {
                    AddCoordinate(Line);
                    AddCoordinate(Column);
                }
                if (AcceptsSymbol) SourceSymbolOptions.Bind(Command, Symbol, Project, ExpectedSnapshot, File, Line, Column);
                switch (name)
                {
                    case "rename":
                        Command.Options.Add(NewName);
                        break;
                    case "add-parameter":
                        Command.Options.Add(Name);
                        Command.Options.Add(ParameterType);
                        Command.Options.Add(Argument);
                        break;
                    case "implement-protocol":
                        Command.Options.Add(Protocol);
                        Command.Options.Add(Members);
                        Command.Options.Add(Forward);
                        break;
                    case "change-signature":
                        Mode.AcceptValues("value", "in", "ref", "out");
                        Command.Options.Add(ParameterType);
                        Command.Options.Add(Mode);
                        Command.Options.Add(CallArguments);
                        Command.Options.Add(Order);
                        break;
                    case "extract-routine":
                    case "extract-variable":
                        Command.Options.Add(Name);
                        AddCoordinate(EndLine);
                        AddCoordinate(EndColumn);
                        break;
                }
            }
            Command.SetAction((result, token) => owner.Execute(this, result, token, name != "apply"));

            void AddCoordinate(Option<int> option)
            {
                ContextCommandOptions.Range(option, 1, int.MaxValue);
                Command.Options.Add(option);
            }
        }
    }
    private readonly Operation[] operations =
        [new("rename"), new("add-parameter"), new("remove-parameter"), new("change-signature"), new("implement-protocol"), new("name-arguments"),
            new("extract-variable"), new("extract-routine"), new("inline-variable"), new("organize-imports"), new("draft"), new("apply")];
    internal Command Command { get; } = new("refactor", """
        Prepare and explicitly apply checked source refactorings in an FPAS project/solution.
        Rename follows resolved source symbols, including their source interface implementations.
        Add Parameter inserts an explicitly typed value parameter and a supplied argument at each direct call.
        Remove Parameter drops an unused value parameter and arguments proven safe to omit.
        Change Signature checks a parameter type, mode or order against the routine and its direct calls.
        Public parameter changes require --closed-context and editable FPAS consumers.
        Implement Protocol reuses compatible members and checks explicit behavior for missing members.
        Name Arguments labels positional arguments at one checked call with repeated input types.
        Extract Variable moves a selected expression into a typed local constant.
        Inline Variable replaces checked local reads with the initializer.
        Extract Routine moves complete statements into a checked nested routine.
        Organize Imports removes safely unused CLR aliases from one source file.
        Draft validates replacement and new source texts and binds them to the project inputs.
        Plans cover analyzed FPAS sources; C#/F# sources, strings and comments are not rewritten.
        Dependency preparation may restore/build. Source bindings are checked before publication.
        Exit codes: 0 success/help, 1 conflict/setup/stale plan, 2 arguments, 130 cancellation.

        Examples:
          fpas refactor rename Example.fpasproj --file main.fpas --line 5 --column 10 --new-name Calculate --plan .temp-data/rename.json

          fpas refactor draft Example.fpasproj --overlay draft.json --plan .temp-data/draft-plan.json

          fpas refactor apply Example.fpasproj --plan .temp-data/rename.json --report json
        """);
    internal RefactorCommand() { foreach (var operation in operations) { operation.Bind(this); Command.Subcommands.Add(operation.Command); } }
    internal bool Owns(Command command) => command == Command || operations.Any(operation => operation.Command == command);
    internal void ReportArgumentError(ParseResult result, string message)
        => RefactorReport.Failure("invalid_argument", message + " See fpas refactor --help.").Write(operations.FirstOrDefault(operation => operation.Command == result.CommandResult.Command)?.Json(result) == true);

    private async Task<int> Execute(Operation operation, ParseResult result, CancellationToken cancellation, bool preparing)
    {
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(operation.Context.Timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellation, deadline.Token); var token = combined.Token;
        RefactorReport? current = null; int exit;
        FileWorkspace? operationWorkspace = null;
        try
        {
            var input = operation.Context.Read(result); var planPath = Path.GetFullPath(result.GetValue(operation.Plan)!);
            if (!Path.GetExtension(planPath).Equals(".json", StringComparison.OrdinalIgnoreCase)) throw new ArgumentException("--plan requires a .json path.");
            RefactorPlan? stored = null; byte[]? storedBytes = null;
            var signature = operation.Command.Name == "change-signature" ? ReadSignature(operation, result) : null;
            if (operation.Command.Name == "rename") _ = IdentifierFacts.Spell(result.GetValue(operation.NewName)!);
            if (operation.Command.Name is "add-parameter" or "extract-variable" or "extract-routine") _ = IdentifierFacts.Spell(result.GetValue(operation.Name)!);
            if (!preparing)
            {
                SourcePublication.ValidateRegularPath(planPath);
                if (new FileInfo(planPath).Length > RefactorPlanJson.MaximumBytes) throw new ContextFailure("invalid_plan", "Plan exceeds 4 MiB.");
                try { storedBytes = await File.ReadAllBytesAsync(planPath, token); stored = RefactorPlanJson.Read(storedBytes); }
                catch (Exception error) when (error is JsonException or ArgumentException) { throw new ContextFailure("invalid_plan", error.Message); }
                FileRefactorPlans.ValidateContext(stored, input.Context, input.Configuration);
            }
            var makingDraft = operation.Command.Name == "draft";
            var manifest = makingDraft ? Path.GetFullPath(result.GetValue(operation.Overlay)!) : null;
            var draft = manifest is null ? null : await AnalysisOverlay.ReadAsync(manifest, token);
            var creations = draft?.Creations ?? stored?.Creations() ?? [];
            operationWorkspace = makingDraft || stored?.Kind == "draft"
                ? await FileWorkspace.OpenDraftAsync(input, token, creations) : await FileWorkspace.OpenAsync(input, token);
            var diagnostics = operationWorkspace.Analysis is { } analysis ? CheckResult.From(analysis).Diagnostics : [];
            current = new(false, "validating", planPath, null, [], false, diagnostics);
            PreparedFileRefactoring prepared;
            if (draft is not null)
            {
                prepared = operationWorkspace.PrepareDraft(draft.Files, creations, token);
                SnapshotGuard.Verify(result, operation.ExpectedSnapshot, prepared.Plan.DraftSnapshotId!);
                await AnalysisOverlay.VerifyAsync(manifest!, draft.Fingerprint, token);
            }
            else if (preparing)
            {
                SnapshotGuard.Verify(result, operation.ExpectedSnapshot, operationWorkspace.Context.SnapshotId);
                prepared = Prepare(operationWorkspace, operation, result, signature, token);
            }
            else prepared = operationWorkspace.ValidateRefactoring(stored!, token);

            current = new(true, preparing ? "planned" : "applied", planPath, prepared.Plan,
                prepared.Plan.Files.Select(file => new RefactorFile(file.Path, false)).ToArray(), false, diagnostics);
            if (current.Bytes().Length > RefactorPlanJson.MaximumBytes) throw new ContextFailure("result_limit", "Refactoring report exceeds 4 MiB; no sources were changed.");
            if (preparing)
            {
                await operationWorkspace.VerifyAsync(token);
                await WritePlan(planPath, RefactorPlanJson.Serialize(prepared.Plan), token);
            }
            else
            {
                await operationWorkspace.ApplyAsync(prepared, async check =>
                {
                    SourcePublication.ValidateRegularPath(planPath);
                    if (!(await File.ReadAllBytesAsync(planPath, check)).AsSpan().SequenceEqual(storedBytes))
                        throw new ContextFailure("inputs_changed", "Refactoring plan changed during application.");
                }, token);
                current = current with { Files = current.Files.Select(file => file with { Written = true }).ToArray(), Diagnostics = [] };
                var checkedResult = CheckResult.From(operationWorkspace.Analysis!);
                current = current with { Diagnostics = checkedResult.Diagnostics };
                current = current with { Success = checkedResult.Success, Rechecked = true, Error = checkedResult.Error };
            }
            exit = current.Success ? 0 : 1;
        }
        catch (SourcePublicationFailure error)
        {
            current = (current ?? RefactorReport.Failure("refactor_failed", error.Message)) with { Success = false, Status = "partial", Error = new("refactor_failed", error.Message),
                Files = current?.Files.Select(file => file with { Written = error.Written.Contains(file.Path) }).ToArray() ?? [],
                Diagnostics = error.Written.Count == 0 ? current?.Diagnostics ?? [] : [], RecoveryBackups = error.Backups }; exit = 1;
        }
        catch (OperationCanceledException) when (token.IsCancellationRequested)
        { current = Fail(cancellation.IsCancellationRequested ? "cancelled" : "timeout", cancellation.IsCancellationRequested ? "Refactoring cancelled." : "Refactoring exceeded --timeout."); exit = cancellation.IsCancellationRequested ? 130 : 1; }
        catch (ContextFailure error)
        {
            current = Fail(error.Code, error.Message);
            var details = AnalysisError.From(error);
            if (!preparing && details.NextAction == "refresh_snapshot")
                details = details with { NextAction = current.Status == "applied" ? "check_sources" : "recreate_plan" };
            current = current with { Error = details };
            exit = error.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1;
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException or JsonException or BadImageFormatException or PlatformNotSupportedException)
        { current = Fail(error is ArgumentException ? "invalid_argument" : "refactor_failed", error.Message); exit = error is ArgumentException ? 2 : 1; }
        return current.Write(operation.Json(result)) ? exit : 1;

        RefactorReport Fail(string code, string message)
        {
            if (current is not null && operationWorkspace is not null)
                current = current with { Files = current.Files.Select(file => file with { Written = operationWorkspace.Written.Contains(file.Path) }).ToArray(),
                    Rechecked = operationWorkspace.Rechecked, Diagnostics = operationWorkspace.Analysis is { } checkedAnalysis ? CheckResult.From(checkedAnalysis).Diagnostics : [] };
            return (current ?? RefactorReport.Failure(code, message)) with
                { Success = false, Status = current?.Files.Any(file => file.Written) == true ? "applied" : code == "cancelled" ? "cancelled" : "failed", Error = new(code, message) };
        }
    }

    private static PreparedFileRefactoring Prepare(FileWorkspace workspace, Operation operation, ParseResult result,
        RefactorSignatureChange? signature, CancellationToken token)
    {
        var file = result.GetValue(operation.File)!;
        var project = result.GetValue(operation.Project);
        if (operation.Command.Name == "organize-imports") return workspace.OrganizeImports(file, project, token);

        var selection = operation.AcceptsSymbol && result.GetValue(operation.Symbol) is { } symbol
            ? new SourceQuery(workspace.Analysis!).SelectDeclaration(project!, symbol)
            : new SourceSelection(file, result.GetValue(operation.Line), result.GetValue(operation.Column), project);
        var closedContext = result.GetResult(operation.ClosedContext) is not null && result.GetValue(operation.ClosedContext);
        return operation.Command.Name switch
        {
            "rename" => workspace.Rename(selection, result.GetValue(operation.NewName)!, token),
            "add-parameter" => workspace.AddParameter(selection, new(result.GetValue(operation.Name)!,
                result.GetValue(operation.ParameterType)!, result.GetValue(operation.Argument)!), token, closedContext),
            "name-arguments" => workspace.NameArguments(selection, token),
            "remove-parameter" => workspace.RemoveParameter(selection, token, closedContext),
            "implement-protocol" => workspace.ImplementProtocol(selection, ReadProtocol(operation, result), token),
            "change-signature" => workspace.ChangeSignature(selection, signature!, token, closedContext),
            "inline-variable" => workspace.InlineVariable(selection, token),
            "extract-routine" => workspace.ExtractRoutine(selection,
                new(result.GetValue(operation.EndLine), result.GetValue(operation.EndColumn)), result.GetValue(operation.Name)!, token),
            "extract-variable" => workspace.ExtractVariable(selection,
                new(result.GetValue(operation.EndLine), result.GetValue(operation.EndColumn)), result.GetValue(operation.Name)!, token),
            _ => throw new InvalidOperationException("Unknown refactoring operation.")
        };
    }

    private static RefactorProtocol ReadProtocol(Operation operation, ParseResult result)
    {
        var members = (result.GetValue(operation.Members) ?? []).Select(text => ParseMember(text, forward: false));
        var forwarded = (result.GetValue(operation.Forward) ?? []).Select(text => ParseMember(text, forward: true));
        return new(result.GetValue(operation.Protocol)!, [.. members, .. forwarded]);

        static RefactorProtocolMember ParseMember(string text, bool forward)
        {
            var split = text.IndexOf('=');
            if (split <= 0 || string.IsNullOrWhiteSpace(text[(split + 1)..]))
                throw new ArgumentException(forward ? "--forward requires MEMBER=METHOD." : "--member requires MEMBER=IMPLEMENTATION.");
            return new(text[..split].Trim(), text[(split + 1)..].Trim(), forward);
        }
    }

    private static RefactorSignatureChange ReadSignature(Operation operation, ParseResult result)
    {
        var type = result.GetValue(operation.ParameterType);
        var mode = result.GetValue(operation.Mode);
        if (result.GetValue(operation.Order) is { } order)
        {
            if (type is not null || mode is not null || result.GetResult(operation.CallArguments) is not null)
                throw new ArgumentException("Use --order without --type, --mode or --argument.");
            var names = order.Split(',', StringSplitOptions.TrimEntries);
            foreach (var name in names) _ = IdentifierFacts.Spell(name);
            if (names.Distinct(StringComparer.Ordinal).Count() != names.Length)
                throw new ArgumentException("--order must list each parameter name once.");
            return new(null, null, [], names);
        }
        if (type is null && mode is null) throw new ArgumentException("Supply --type, --mode, or --order.");
        var replacements = (result.GetValue(operation.CallArguments) ?? []).Select(ParseArgument).ToArray();
        return new(type, mode, replacements);

        static RefactorArgumentOverride ParseArgument(string text)
        {
            var match = Regex.Match(text, @"\A(?:(?<line>[0-9]+):(?<column>[0-9]+)=|(?<file>.+?):(?<line>[0-9]+):(?<column>[0-9]+)=)(?<expression>[\s\S]*)\z",
                RegexOptions.CultureInvariant | RegexOptions.NonBacktracking);
            if (!match.Success ||
                !int.TryParse(match.Groups["line"].ValueSpan, NumberStyles.None, CultureInfo.InvariantCulture, out var line) || line < 1 ||
                !int.TryParse(match.Groups["column"].ValueSpan, NumberStyles.None, CultureInfo.InvariantCulture, out var column) || column < 1 ||
                string.IsNullOrWhiteSpace(match.Groups["expression"].Value))
                throw new ArgumentException("--argument requires [FILE:]LINE:COLUMN=EXPRESSION with positive coordinates.");
            var file = match.Groups["file"];
            return new(new(line, column), match.Groups["expression"].Value, file.Success ? Path.GetFullPath(file.Value) : null);
        }
    }

    private static async Task WritePlan(string path, byte[] bytes, CancellationToken token)
    {
        var directory = Path.GetDirectoryName(path)!; Directory.CreateDirectory(directory); SourcePublication.ValidateRegularPath(directory);
        var temporary = Path.Combine(directory, ".fpas-plan-" + Guid.NewGuid().ToString("N"));
        try { await File.WriteAllBytesAsync(temporary, bytes, token); token.ThrowIfCancellationRequested(); File.Move(temporary, path, overwrite: false); }
        finally { File.Delete(temporary); }
    }
}
