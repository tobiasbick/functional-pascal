using System.CommandLine;

namespace Fpas.Cli.Projects;

internal sealed class InitCommand
{
    private sealed class Template(string kind, Option<string> report)
    {
        internal Command Command { get; } = new(kind, $"Create an FPAS {kind}, preserving existing files.\n\nExamples:\n  fpas init {kind} Example --name Example\n");
        internal Argument<string> Path { get; } = new("path") { Description = "Destination directory." };
        internal Option<string> Name { get; } = new("--name") { Description = "Qualified Pascal name; defaults to the directory name." };
        internal Option<string> Version { get; } = new("--sdk-version") { Description = "Explicit Fpas.NET.Sdk package version; defaults to this tool's SDK." };
        internal Option<string> Feed { get; } = new("--sdk-feed") { Description = "Optional SDK package feed (local directory or HTTPS URL)." };
        internal Option<string[]> Projects { get; } = new("--project") { Description = "Existing project to include; repeat for a solution.", AllowMultipleArgumentsPerToken = false };
        internal Option<bool> DryRun { get; } = new("--dry-run") { Description = "Preview the exact file plan without writing." };
        internal Option<string> Framework { get; } = new("--framework") { Description = "Native test framework: xunit, mstest or nunit.", DefaultValueFactory = _ => "xunit" };

        internal void Bind()
        {
            Command.Arguments.Add(Path);
            foreach (var option in new Option[] { Name, DryRun }) Command.Options.Add(option);
            if (kind == "solution") Command.Options.Add(Projects);
            else { Command.Options.Add(Version); Command.Options.Add(Feed); }
            if (kind == "test") { Framework.AcceptOnlyFromAmong("xunit", "mstest", "nunit"); Command.Options.Add(Framework); }
            Command.SetAction((result, _) =>
            {
                var json = Json(result);
                try
                {
                    var report = Scaffolder.Create(kind, result.GetValue(Path)!, result.GetValue(Name), kind == "solution" ? null : result.GetValue(Version),
                        kind == "solution" ? null : result.GetValue(Feed), kind == "solution" ? result.GetValue(Projects) ?? [] : [], result.GetValue(DryRun),
                        kind == "test" ? result.GetValue(Framework)! : "xunit");
                    report.Write(json); return Task.FromResult(report.Success ? 0 : 1);
                }
                catch (Exception exception) when (exception is ArgumentException or IOException or UnauthorizedAccessException or System.Xml.XmlException)
                {
                    Failure(exception.Message, json, exception is ArgumentException ? "invalid_argument" : "init_failed");
                    return Task.FromResult(exception is ArgumentException ? 2 : 1);
                }
            });
        }

        internal bool Json(ParseResult result) => result.GetResult(report)?.Tokens.Any(token => token.Value == "json") == true;
        internal void Failure(string message, bool json, string code)
            => new ProjectReport("init " + kind, false, "failed", null) { Error = new(code, message) }.Write(json);
    }

    internal Command Command { get; } = new("init", """
        Create a project, library, test project or solution with an explicit file plan.
        Existing differing files are conflicts; identical files remain unchanged.

        Examples:
          fpas init project Hello --name Hello

          fpas init library Math --name Demo.Math

          fpas init test Tests --framework xunit

          fpas init solution Demo --project Hello/Hello.fpasproj
        """);
    private readonly Option<string> report = new("--report") { Description = "Write one structured JSON result.", Recursive = true };
    private readonly Template[] templates;

    internal InitCommand()
    {
        report.AcceptOnlyFromAmong("json"); Command.Options.Add(report);
        templates = [new("project", report), new("library", report), new("test", report), new("solution", report)];
        foreach (var template in templates) { template.Bind(); Command.Subcommands.Add(template.Command); }
    }
    internal bool Owns(Command command) => command == Command || templates.Any(template => template.Command == command);
    internal void ReportArgumentError(ParseResult result, string message)
    {
        var template = templates.FirstOrDefault(template => template.Command == result.CommandResult.Command);
        if (template is not null) template.Failure(message, template.Json(result), "invalid_argument");
        else new ProjectReport("init", false, "failed", null) { Error = new("invalid_argument", message + " See fpas init --help.") }
            .Write(result.GetResult(report)?.Tokens.Any(token => token.Value == "json") == true);
    }
}
