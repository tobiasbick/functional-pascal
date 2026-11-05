using Fpas.Cli.Analysis;
using Fpas.Compiler;
using Fpas.Compiler.Formatting;
using Fpas.Workspaces.Formatting;
using Fpas.Workspaces.Loading;
using System.CommandLine;
using System.Text;
using System.Text.Json;

namespace Fpas.Cli.Formatting;

internal sealed class FormatCommand
{
    private readonly Argument<string[]> inputs = new("paths") { Arity = ArgumentArity.OneOrMore, Description = "Explicit .fpas files, directories, .fpasproj or .slnx inputs." };
    private readonly Option<bool> check = new("--check") { Description = "Validate formatting without writing; exit 2 if changes are needed." };
    private readonly Option<bool> list = new("--list") { Description = "With --check, list changed absolute paths on stdout." };
    private readonly Option<bool> stdout = new("--stdout") { Description = "Print one formatted source without writing files." };
    private readonly Option<string> report = new("--report") { Description = "Write one structured JSON result to stdout." };
    private readonly Option<string> configuration = new("--configuration") { DefaultValueFactory = _ => "Debug", Description = "Configuration for evaluated project/solution Compile inputs." };
    private readonly Option<int> timeout = new("--timeout") { DefaultValueFactory = _ => 120, Description = "Overall deadline in seconds (1..3600)." };

    internal Command Command { get; } = new("fmt", """
        Format FPAS with one fixed style, preserving tokens and comments.
        Sources are parsed without binding or compilation. Project inputs use evaluated Compile items.
        Directories recurse, excluding .git, .temp-data, bin, obj, node_modules and symbolic links.
        Exit codes: 0 success/help, 1 failure/deadline, 2 check changes/invalid arguments, 130 cancellation.

        Examples:
          fpas fmt examples/hello/main.fpas

          fpas fmt --check --list examples

          fpas fmt --stdout examples/hello/main.fpas

          fpas fmt --check examples/hello/Hello.fpasproj --report json
        """);

    internal FormatCommand()
    {
        report.AcceptOnlyFromAmong("json");
        ContextCommandOptions.Range(timeout, 1, 3600);
        Command.Arguments.Add(inputs);
        foreach (var option in new Option[] { check, list, stdout, report, configuration, timeout }) Command.Options.Add(option);
        Command.SetAction(Execute);
    }

    internal void ReportArgumentError(ParseResult result, string message)
        => new FormatReport(false, Mode(result), [], [], new("invalid_argument", message + " See fpas fmt --help."))
            .Write(Json(result), false);

    internal void Validate(ParseResult result) => CommandArguments.Validate(result, inputs);

    private bool Json(ParseResult result) => result.GetResult(report)?.Tokens.Any(token => token.Value == "json") == true;
    private string Mode(ParseResult result) => result.GetValue(stdout) ? "stdout" : result.GetValue(check) ? "check" : "write";

    private async Task<int> Execute(ParseResult result, CancellationToken cancellationToken)
    {
        var mode = Mode(result); var json = Json(result);
        if (result.GetValue(list) && !result.GetValue(check) || mode == "stdout" && (result.GetValue(check) || json) || json && result.GetValue(list))
        { ReportArgumentError(result, "--list requires --check; --stdout cannot combine with --check/--report, and --list cannot combine with --report."); return 2; }
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, deadline.Token);
        var token = combined.Token;
        var files = new List<FormatFile>();
        var diagnostics = new List<CompilerDiagnostic>();
        FormatReport completed; int exit;
        try
        {
            var paths = await FormatSources.CollectAsync(result.GetValue(inputs)!, result.GetValue(configuration)!, token);
            if (mode == "stdout" && paths.Count != 1) throw new ContextFailure("invalid_argument", "--stdout requires exactly one selected FPAS source.");
            var snapshots = new List<FormatSnapshot>();
            long bytes = 0;
            foreach (var path in paths)
            {
                token.ThrowIfCancellationRequested();
                if (new FileInfo(path).Length > 64 * 1024 * 1024 - bytes) throw new IOException("Format inputs exceed the 64 MiB request limit.");
                var original = await File.ReadAllBytesAsync(path, token);
                if ((bytes += original.Length) > 64 * 1024 * 1024) throw new IOException("Format inputs exceed the 64 MiB request limit.");
                var text = FormatPublication.Utf8.GetString(original);
                var formatted = FpasFormatter.Format(new(path, text), token);
                diagnostics.AddRange(formatted.Diagnostics);
                if (formatted.Success) snapshots.Add(new(path, original, formatted.Text!));
            }
            foreach (var snapshot in snapshots) files.Add(new(snapshot.Path, snapshot.Changed, false));
            if (diagnostics.Count != 0) { completed = new(false, mode, files, diagnostics); exit = 1; }
            else
            {
                await FormatPublication.Verify(snapshots, token);
                if (mode == "stdout") await Console.Out.WriteAsync(snapshots[0].Formatted.AsMemory(), token);
                if (mode == "write")
                    for (var index = 0; index < snapshots.Count; index++)
                        if (files[index].Changed)
                        {
                            await FormatPublication.Write(snapshots[index], token);
                            files[index] = files[index] with { Written = true };
                        }
                var drift = mode == "check" && files.Any(file => file.Changed);
                completed = new(!drift, mode, files, diagnostics); exit = drift ? 2 : 0;
            }
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            var cancelled = cancellationToken.IsCancellationRequested;
            completed = new(false, mode, files, diagnostics, new(cancelled ? "cancelled" : "timeout", cancelled ? "Formatting cancelled." : "Formatting exceeded --timeout."));
            exit = cancelled ? 130 : 1;
        }
        catch (ContextFailure failure) { completed = new(false, mode, files, diagnostics, new(failure.Code, failure.Message)); exit = failure.Code == "invalid_argument" ? 2 : 1; }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException or JsonException or ArgumentException or PlatformNotSupportedException or System.ComponentModel.Win32Exception or System.Xml.XmlException)
        { completed = new(false, mode, files, diagnostics, new("format_failed", exception.Message)); exit = 1; }
        completed.Write(json, result.GetValue(list));
        return exit;
    }
}
