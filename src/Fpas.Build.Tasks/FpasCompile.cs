using Fpas.Build.Tasks.Incremental;
using Fpas.Build.Tasks.Documentation;
using System.Diagnostics;
using System.Text.Json;
using Microsoft.Build.Framework;

namespace Fpas.Build.Tasks;

public sealed class FpasCompile : Microsoft.Build.Utilities.Task, ICancelableTask
{
    [Required] public ITaskItem[] Sources { get; set; } = [];
    [Required] public ITaskItem[] References { get; set; } = [];
    [Required] public string CompilerPath { get; set; } = "";
    [Required] public string OutputAssembly { get; set; } = "";
    [Required] public string CacheFile { get; set; } = "";
    public string DotnetHostPath { get; set; } = "";
    public string OutputType { get; set; } = "Exe";
    public string Configuration { get; set; } = "Debug";
    public string TargetFramework { get; set; } = "net10.0";
    public string DebugType { get; set; } = "portable";
    public bool Optimize { get; set; }
    public string DocumentationFile { get; set; } = "";
    public string DocumentationDestination { get; set; } = "";
    public string DocumentationOutput { get; set; } = "";
    public bool TreatWarningsAsErrors { get; set; }
    public string WarningsAsErrors { get; set; } = "";
    public string WarningsNotAsErrors { get; set; } = "";
    public string NoWarn { get; set; } = "";
    [Output] public ITaskItem[] WrittenDocumentation { get; set; } = [];
    private readonly List<JsonElement> compilerDiagnostics = [];
    private readonly CancellationTokenSource cancellation = new();

    public override bool Execute()
    {
        if (Sources.Length == 0 || TargetFramework != "net10.0" || OutputType is not ("Exe" or "Library") || DebugType is not ("portable" or "none"))
        {
            Error("FP5001", "The FPAS SDK requires sources, net10.0, OutputType Exe or Library, and DebugType portable or none.");
            return false;
        }
        try
        {
            var sources = Sources.Select(item => item.GetMetadata("FullPath")).ToArray();
            var references = References.Select(item => item.GetMetadata("FullPath")).ToArray();
            var output = Path.GetFullPath(OutputAssembly);
            var cache = Path.GetFullPath(CacheFile);
            var symbols = DebugType == "portable" ? Path.ChangeExtension(output, ".pdb") : null;
            var documents = string.IsNullOrEmpty(DocumentationFile) ? [] : DocumentationOutputs.Paths(DocumentationFile, DocumentationDestination, DocumentationOutput);
            if (documents.Any(path => !Path.GetExtension(path).Equals(".xml", StringComparison.OrdinalIgnoreCase)))
            {
                Error("FP5001", "DocumentationFile must name an .xml file."); return false;
            }
            if (documents.Any(path => sources.Concat(references).Append(output).Append(cache).Append(cache + ".arguments.json").Contains(path, DocumentationOutputs.Comparer)))
            {
                Error("FP5001", "XML documentation output paths must differ from source, reference, assembly, and compiler-cache paths."); return false;
            }
            var previous = CompileFingerprint.Read(cache);
            var fingerprint = CompileFingerprint.Compute(sources, references, CompilerPath,
                $"{OutputType}\0{Configuration}\0{TargetFramework}\0{output}\0{DebugType}\0{Optimize}\0{string.Join('\0', documents)}\0{TreatWarningsAsErrors}\0{WarningsAsErrors}\0{WarningsNotAsErrors}\0{NoWarn}");
            if (previous?.IsCurrent(fingerprint, output, symbols, documents) == true &&
                (OutputType == "Library" || File.Exists(Path.ChangeExtension(output, ".runtimeconfig.json"))))
            {
                foreach (var diagnostic in previous.Diagnostics ?? []) ReportDiagnostic(diagnostic);
                WrittenDocumentation = documents.Select(path => (ITaskItem)new Microsoft.Build.Utilities.TaskItem(path)).ToArray();
                Log.LogMessage(MessageImportance.Low, "FPAS compilation is up to date.");
                return !Log.HasLoggedErrors;
            }
            Directory.CreateDirectory(Path.GetDirectoryName(cache)!);
            var arguments = new List<string>(sources);
            arguments.AddRange(["--output", output, "--target", OutputType == "Library" ? "library" : "exe", "--diagnostics", "json"]);
            arguments.AddRange(["--debug", DebugType]);
            if (Optimize) arguments.Add("--optimize");
            if (documents.Length != 0) arguments.AddRange(["--documentation", documents[0]]);
            if (TreatWarningsAsErrors) arguments.Add("--warn-as-error");
            if (!string.IsNullOrWhiteSpace(WarningsAsErrors)) arguments.AddRange(["--warnings-as-errors", WarningsAsErrors]);
            if (!string.IsNullOrWhiteSpace(WarningsNotAsErrors)) arguments.AddRange(["--warnings-not-as-errors", WarningsNotAsErrors]);
            if (!string.IsNullOrWhiteSpace(NoWarn)) arguments.AddRange(["--no-warn", NoWarn]);
            foreach (var reference in references)
                arguments.AddRange(["--reference", reference]);
            var response = cache + ".arguments.json";
            File.WriteAllText(response, JsonSerializer.Serialize(arguments));
            Log.LogMessage(MessageImportance.High, "FPAS: compiling {0} source file(s)", sources.Length);
            if (!RunCompiler(response))
                return false;
            if (!File.Exists(output) || symbols is not null && !File.Exists(symbols) || documents.Length != 0 && !File.Exists(documents[0]))
            {
                Error("FP5003", "The compiler did not produce the requested assembly.");
                return false;
            }
            DocumentationOutputs.Publish(documents, previous?.DocumentationFiles ?? []);
            WrittenDocumentation = documents.Select(path => (ITaskItem)new Microsoft.Build.Utilities.TaskItem(path)).ToArray();
            CompileFingerprint.Save(cache, fingerprint, output, symbols, documents, compilerDiagnostics.ToArray());
            return true;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or JsonException or System.ComponentModel.Win32Exception)
        {
            Error("FP5002", exception.Message);
            return false;
        }
    }

    private bool RunCompiler(string response)
    {
        var host = string.IsNullOrEmpty(DotnetHostPath)
            ? Environment.GetEnvironmentVariable("DOTNET_HOST_PATH") ?? "dotnet" : DotnetHostPath;
        var start = new ProcessStartInfo(host) { UseShellExecute = false, RedirectStandardOutput = true, RedirectStandardError = true };
        foreach (var argument in new[] { Path.GetFullPath(CompilerPath), "compile", "--response-file", response })
            start.ArgumentList.Add(argument);
        using var process = Process.Start(start) ?? throw new IOException("Unable to start the FPAS compiler.");
        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellation.Token);
        timeout.CancelAfter(TimeSpan.FromMinutes(2));
        var stdout = process.StandardOutput.ReadToEndAsync(timeout.Token);
        var stderr = process.StandardError.ReadToEndAsync(timeout.Token);
        try { process.WaitForExitAsync(timeout.Token).GetAwaiter().GetResult(); }
        catch (OperationCanceledException)
        {
            if (!process.HasExited)
                process.Kill(entireProcessTree: true);
            process.WaitForExit();
            Error("FP5004", "FPAS compilation was canceled or exceeded its time limit.");
            return false;
        }
        var text = stdout.GetAwaiter().GetResult();
        var error = stderr.GetAwaiter().GetResult();
        using var result = JsonDocument.Parse(text);
        foreach (var diagnostic in result.RootElement.GetProperty("diagnostics").EnumerateArray())
        {
            compilerDiagnostics.Add(diagnostic.Clone()); ReportDiagnostic(diagnostic);
        }
        if (process.ExitCode != 0 || !result.RootElement.GetProperty("success").GetBoolean())
        {
            if (!Log.HasLoggedErrors)
                Error("FP5003", "Compiler failed: " + error);
            return false;
        }
        return !Log.HasLoggedErrors;
    }

    public void Cancel() => cancellation.Cancel();
    private void ReportDiagnostic(JsonElement diagnostic)
    {
        var location = diagnostic.GetProperty("location");
        if (diagnostic.TryGetProperty("severity", out var severity) && severity.GetString() == "warning")
            Log.LogWarning(null, diagnostic.GetProperty("code").GetString(), null, location.GetProperty("path").GetString(),
                location.GetProperty("line").GetInt32(), location.GetProperty("column").GetInt32(), 0, 0, diagnostic.GetProperty("message").GetString());
        else Log.LogError(null, diagnostic.GetProperty("code").GetString(), null, location.GetProperty("path").GetString(),
            location.GetProperty("line").GetInt32(), location.GetProperty("column").GetInt32(), 0, 0, diagnostic.GetProperty("message").GetString() ?? "FPAS compilation failed.");
    }
    private void Error(string code, string message) => Log.LogError(null, code, null, null, 0, 0, 0, 0, message);
}
