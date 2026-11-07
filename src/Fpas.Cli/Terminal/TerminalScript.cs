namespace Fpas.Cli.Terminal;

[System.Text.Json.Serialization.JsonUnmappedMemberHandling(System.Text.Json.Serialization.JsonUnmappedMemberHandling.Disallow)]
internal sealed record TerminalStep(string Action, string? Text = null, int? Columns = null, int? Rows = null, int? ExitCode = null);
[System.Text.Json.Serialization.JsonUnmappedMemberHandling(System.Text.Json.Serialization.JsonUnmappedMemberHandling.Disallow)]
internal sealed record TerminalScript(string Executable, string[] Arguments, TerminalStep[] Steps, int SchemaVersion = 1,
    string? Directory = null, int Columns = 80, int Rows = 24, int TimeoutSeconds = 30, int OutputBytes = 1_048_576);
internal sealed record TerminalResult(int ProcessId, int? ExitCode, int CompletedSteps, string Transcript, bool Truncated);
