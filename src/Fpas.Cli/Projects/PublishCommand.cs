using System.CommandLine;
using System.Text.Json;
using Fpas.Cli.Reporting;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Projects;

namespace Fpas.Cli.Projects;

internal sealed class PublishCommand
{
    internal Command Command { get; } = new("publish", """
        Publish an executable project, standalone source or selected solution application.
        Native AOT is the default. Builds require the target platform's native toolchain.
        Output must be empty or an unchanged directory previously published by fpas.

        Examples:
          fpas publish App/App.fpasproj --output artifacts/app --runtime win-x64
          fpas publish App.slnx --project App/App.fpasproj --output artifacts/app --mode single-file
          fpas publish main.fpas --sdk-feed ./feed --output artifacts/app --configuration Debug --report json
        """);
    private readonly Argument<string> input = new("input");
    private readonly Option<string> output = new("--output") { Required = true, Description = "Owned publish directory, outside source and build inputs." };
    private readonly Option<string> project = new("--project") { Description = "Included executable project when a solution is ambiguous." };
    private readonly Option<string> configuration = new("--configuration") { DefaultValueFactory = _ => "Release" };
    private readonly Option<string> mode = new("--mode") { DefaultValueFactory = _ => "aot", Description = "aot, single-file, self-contained, apphost or framework-dependent." };
    private readonly Option<string> runtime = new("--runtime") { Description = "win-x64 or linux-x64; defaults to the host." };
    private readonly Option<bool> noRestore = new("--no-restore");
    private readonly Option<bool> noBuild = new("--no-build");
    private readonly Option<bool> dryRun = new("--dry-run") { Description = "Evaluate selection and validate output without publishing." };
    private readonly Option<string> key = new("--key-file") { Description = "RSA .snk key; enables SignAssembly for the project graph." };
    private readonly Option<bool> publicSign = new("--public-sign");
    private readonly Option<bool> delaySign = new("--delay-sign");
    private readonly Option<string> certificate = new("--certificate") { Description = "PFX certificate for a CMS signature over all delivered files." };
    private readonly Option<string> passwordEnvironment = new("--certificate-password-env") { Description = "Environment variable containing the PFX password; no password is passed on the command line." };
    private readonly Option<string> version = new("--sdk-version");
    private readonly Option<string> feed = new("--sdk-feed");
    private readonly Option<int> timeout = new("--timeout") { DefaultValueFactory = _ => 600, Description = "Whole-operation deadline, 1..3600 seconds." };
    private readonly Option<string> report = new("--report") { Description = "Write one JSON report." };

    internal PublishCommand()
    {
        report.AcceptOnlyFromAmong("json"); Fpas.Cli.Analysis.ContextCommandOptions.Range(timeout, 1, 3600);
        mode.AcceptOnlyFromAmong("aot", "single-file", "self-contained", "apphost", "framework-dependent");
        Command.Arguments.Add(input);
        foreach (var option in new Option[] { output, project, configuration, mode, runtime, noRestore, noBuild, dryRun, key, publicSign, delaySign, certificate, passwordEnvironment, version, feed, timeout, report })
            Command.Options.Add(option);
        Command.SetAction(Execute);
    }

    private bool Json(ParseResult result) => result.GetResult(report)?.Tokens.Any(token => token.Value == "json") == true;
    internal void ArgumentError(ParseResult result, string message) =>
        new ProjectReport("publish", false, "failed", null) { Error = new("invalid_argument", message + " See fpas publish --help.") }.Write(Json(result));

    private async Task<int> Execute(ParseResult result, CancellationToken cancellation)
    {
        var json = Json(result);
        string? context = null;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(result.GetValue(timeout)));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellation, deadline.Token);
        try
        {
            context = Path.GetFullPath(result.GetValue(input)!);
            var options = new PublishOptions(result.GetValue(mode)!, result.GetValue(runtime), result.GetValue(configuration)!,
                result.GetValue(noRestore), result.GetValue(noBuild), result.GetValue(key), result.GetValue(publicSign), result.GetValue(delaySign),
                result.GetValue(certificate), result.GetValue(passwordEnvironment));
            options.Validate();
            var destination = Path.GetFullPath(result.GetValue(output)!);
            if (Path.GetPathRoot(destination) == destination) throw new ArgumentException("Output cannot be a filesystem root.");
            var comparison = OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal;
            if (Path.GetDirectoryName(context)!.StartsWith(Path.TrimEndingDirectorySeparator(destination) + Path.DirectorySeparatorChar, comparison) ||
                string.Equals(Path.GetDirectoryName(context), destination, comparison))
                throw new ArgumentException("Output must not contain the source project.");
            await PublishDirectory.Check(destination, combined.Token);
            var source = Path.GetExtension(context).Equals(".fpas", StringComparison.OrdinalIgnoreCase);
            if (!source && (result.GetValue(version) is not null || result.GetValue(feed) is not null))
                throw new ArgumentException("--sdk-version and --sdk-feed apply only to standalone sources.");
            if (source && result.GetValue(dryRun)) throw new ArgumentException("Use a project for --dry-run; standalone source preparation writes a project.");
            var buildContext = source ? new SourceProject(context, ToolSdk.ValidateVersion(result.GetValue(version)), result.GetValue(feed)).Project : context;
            var selected = await ProjectDriver.SelectExecutableAsync(buildContext, options.Configuration, result.GetValue(project), combined.Token);
            PublishResult? published = result.GetValue(dryRun) ? null : await PublishDirectory.Publish(selected, options, destination, combined.Token,
                buildContext, ProjectProgress.BuildOutput(json, "Publishing", context));
            var success = published is null || published.Build.ExitCode == 0 && !published.Build.Truncated;
            new ProjectReport("publish", success, success ? published is null ? "preview" : "ok" : "publish_failed", context)
            {
                Target = selected, Build = published?.Build, DryRun = published is null,
                Publish = new(options.Mode, options.RuntimeIdentifier, destination, published?.Files ?? []),
                Error = success ? null : new("publish_failed", "Publish failed or exceeded the output limit; the previous output was preserved.")
            }.Write(json);
            if (success && !json) Console.WriteLine(destination);
            return success ? 0 : 1;
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        { Failure(cancellation.IsCancellationRequested ? "cancelled" : "timeout", "Publishing cancelled or exceeded --timeout."); return cancellation.IsCancellationRequested ? 130 : 1; }
        catch (ContextFailure error) { Failure(error.Code, error.Message, error.Candidates); return error.Code is "invalid_argument" or "ambiguous_project" or "not_executable" ? 2 : 1; }
        catch (System.Security.Cryptography.CryptographicException)
        { Failure("publish_failed", "Delivery signing failed; check the certificate and password environment variable."); return 1; }
        catch (Exception error) when (error is ArgumentException or IOException or UnauthorizedAccessException or JsonException or System.Xml.XmlException or System.ComponentModel.Win32Exception)
        { Failure(error is ArgumentException ? "invalid_argument" : "publish_failed", error.Message); return error is ArgumentException ? 2 : 1; }

        void Failure(string code, string message, IReadOnlyList<string>? candidates = null) =>
            new ProjectReport("publish", false, "failed", context) { Error = new(code, message, candidates) }.Write(json);
    }
}
