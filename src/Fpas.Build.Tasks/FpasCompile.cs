// [Documentation](../../docs/build/msbuild.md#sdk-task-and-incremental-state)
using Fpas.Build.Tasks.Incremental;
using Fpas.Build.Tasks.Documentation;
using System.Diagnostics;
using System.Text.Json;
using Microsoft.Build.Framework;

namespace Fpas.Build.Tasks;

/// <summary>Compiles FPAS sources through the FPAS compiler with incremental fingerprinting.</summary>
public sealed class FpasCompile : Microsoft.Build.Utilities.Task, ICancelableTask
{
    /// <summary>Gets or sets the FPAS source items to compile.</summary>
    [Required] public ITaskItem[] Sources { get; set; } = [];
    /// <summary>Gets or sets the resolved CLR assembly references.</summary>
    [Required] public ITaskItem[] References { get; set; } = [];
    /// <summary>Gets or sets neutral manifest resources and their original source paths.</summary>
    public ITaskItem[] Resources { get; set; } = [];
    /// <summary>Gets or sets the compiler executable or managed entry assembly.</summary>
    [Required] public string CompilerPath { get; set; } = "";
    /// <summary>Gets or sets the compiler assembly used to verify toolchain identity.</summary>
    public string CompilerAssemblyPath { get; set; } = "";
    /// <summary>Gets or sets the destination assembly path.</summary>
    [Required] public string OutputAssembly { get; set; } = "";
    /// <summary>Gets or sets the incremental compilation proof path.</summary>
    [Required] public string CacheFile { get; set; } = "";
    /// <summary>Gets or sets the dotnet host, or empty to use the configured environment.</summary>
    public string DotnetHostPath { get; set; } = "";
    /// <summary>Gets or sets Exe or Library output.</summary>
    public string OutputType { get; set; } = "Exe";
    /// <summary>Gets or sets the build configuration included in the compilation fingerprint.</summary>
    public string Configuration { get; set; } = "Debug";
    /// <summary>Gets or sets the supported target framework, net10.0.</summary>
    public string TargetFramework { get; set; } = "net10.0";
    /// <summary>Gets or sets portable symbols or none.</summary>
    public string DebugType { get; set; } = "portable";
    /// <summary>Gets or sets whether compiler optimizations are enabled.</summary>
    public bool Optimize { get; set; }
    /// <summary>Gets or sets the optional assembly signing key path.</summary>
    public string SigningKey { get; set; } = "";
    /// <summary>Gets or sets public signing without a private-key signature.</summary>
    public bool PublicSign { get; set; }
    /// <summary>Gets or sets whether signing reserves a signature for later completion.</summary>
    public bool DelaySign { get; set; }
    /// <summary>Gets or sets the optional XML documentation path.</summary>
    public string DocumentationFile { get; set; } = "";
    /// <summary>Gets or sets the optional documentation publication destination.</summary>
    public string DocumentationDestination { get; set; } = "";
    /// <summary>Gets or sets the optional documentation output copy.</summary>
    public string DocumentationOutput { get; set; } = "";
    /// <summary>Gets or sets whether all enabled warnings are errors.</summary>
    public bool TreatWarningsAsErrors { get; set; }
    /// <summary>Gets or sets diagnostic codes promoted to errors.</summary>
    public string WarningsAsErrors { get; set; } = "";
    /// <summary>Gets or sets diagnostic codes exempt from warning promotion.</summary>
    public string WarningsNotAsErrors { get; set; } = "";
    /// <summary>Gets or sets diagnostic codes to suppress.</summary>
    public string NoWarn { get; set; } = "";
    /// <summary>Gets the XML documentation files written or confirmed current.</summary>
    [Output] public ITaskItem[] WrittenDocumentation { get; set; } = [];
    private readonly List<JsonElement> compilerDiagnostics = [];
    private readonly object cancellationGate = new();
    private readonly CancellationTokenSource cancellation = new();
    private bool completed;

    /// <inheritdoc/>
    public override bool Execute()
    {
        try { return ExecuteCore(); }
        finally
        {
            lock (cancellationGate) { completed = true; cancellation.Dispose(); }
        }
    }

    private bool ExecuteCore()
    {
        if (Sources.Length == 0 || TargetFramework != "net10.0" || OutputType is not ("Exe" or "WinExe" or "Library") || DebugType is not ("portable" or "none"))
        {
            Error("FP5001", "The FPAS SDK requires sources, net10.0, OutputType Exe, WinExe or Library, and DebugType portable or none.");
            return false;
        }
        try
        {
            var sources = Sources.Select(item => item.GetMetadata("FullPath")).ToArray();
            var references = References.Select(item => item.GetMetadata("FullPath")).ToArray();
            var resources = ResourceInput.Read(Resources);
            var output = Path.GetFullPath(OutputAssembly);
            var cache = Path.GetFullPath(CacheFile);
            var symbols = DebugType == "portable" ? Path.ChangeExtension(output, ".pdb") : null;
            var documents = string.IsNullOrEmpty(DocumentationFile) ? [] : DocumentationOutputs.Paths(DocumentationFile, DocumentationDestination, DocumentationOutput);
            if (documents.Any(path => !Path.GetExtension(path).Equals(".xml", StringComparison.OrdinalIgnoreCase)))
            {
                Error("FP5001", "DocumentationFile must name an .xml file."); return false;
            }
            if (documents.Any(path => sources.Concat(references).Concat(resources.SelectMany(resource => new[] { resource.Path, resource.SourcePath }))
                .Append(output).Append(cache).Append(cache + ".arguments.json").Contains(path, DocumentationOutputs.Comparer)))
            {
                Error("FP5001", "XML documentation output paths must differ from source, reference, resource, assembly, and compiler-cache paths."); return false;
            }
            var previous = CompileFingerprint.Read(cache);
            var fingerprint = CompileFingerprint.Compute(sources, references, CompilerPath,
                CompileFingerprint.Options(OutputType, Configuration, TargetFramework, output, DebugType, Optimize, documents,
                    TreatWarningsAsErrors, WarningsAsErrors, WarningsNotAsErrors, NoWarn) + SigningInputs.Identity(SigningKey, PublicSign, DelaySign), CompilerAssemblyPath, resources);
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
            if (SigningKey.Length != 0) arguments.AddRange(["--key-file", SigningKey]);
            if (PublicSign) arguments.Add("--public-sign");
            if (DelaySign) arguments.Add("--delay-sign");
            if (CompilerAssemblyPath.Length > 0)
            {
                using var compiler = File.OpenRead(CompilerAssemblyPath);
                arguments.AddRange(["--expected-compiler-sha256", Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(compiler))]);
            }
            if (Optimize) arguments.Add("--optimize");
            if (documents.Length != 0) arguments.AddRange(["--documentation", documents[0]]);
            if (TreatWarningsAsErrors) arguments.Add("--warn-as-error");
            if (!string.IsNullOrWhiteSpace(WarningsAsErrors)) arguments.AddRange(["--warnings-as-errors", WarningsAsErrors]);
            if (!string.IsNullOrWhiteSpace(WarningsNotAsErrors)) arguments.AddRange(["--warnings-not-as-errors", WarningsNotAsErrors]);
            if (!string.IsNullOrWhiteSpace(NoWarn)) arguments.AddRange(["--no-warn", NoWarn]);
            foreach (var reference in references)
                arguments.AddRange(["--reference", reference]);
            foreach (var resource in resources)
                arguments.AddRange(["--embedded-resource", JsonSerializer.Serialize(new[] { resource.Name, resource.Path })]);
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
        var managed = CompilerPath.EndsWith(".dll", StringComparison.OrdinalIgnoreCase);
        var start = new ProcessStartInfo(managed ? host : Path.GetFullPath(CompilerPath)) { UseShellExecute = false, RedirectStandardOutput = true, RedirectStandardError = true };
        if (managed) start.ArgumentList.Add(Path.GetFullPath(CompilerPath));
        foreach (var argument in new[] { "compile", "--response-file", response })
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
        JsonDocument result;
        try { result = JsonDocument.Parse(text); }
        catch (JsonException)
        {
            // A crashed compiler leaves no JSON result; its stderr explains the failure.
            Error("FP5003", "Compiler failed (exit code " + process.ExitCode + "): " + error);
            return false;
        }
        using var owned = result;
        if (result.RootElement.ValueKind != JsonValueKind.Object ||
            !result.RootElement.TryGetProperty("diagnostics", out var diagnostics) || diagnostics.ValueKind != JsonValueKind.Array ||
            !result.RootElement.TryGetProperty("success", out var succeeded) || succeeded.ValueKind is not (JsonValueKind.True or JsonValueKind.False))
        {
            Error("FP5003", "The compiler returned an invalid JSON result: " + error);
            return false;
        }
        foreach (var diagnostic in diagnostics.EnumerateArray())
        {
            compilerDiagnostics.Add(diagnostic.Clone()); ReportDiagnostic(diagnostic);
        }
        if (process.ExitCode != 0 || !succeeded.GetBoolean())
        {
            if (!Log.HasLoggedErrors)
                Error("FP5003", "Compiler failed: " + error);
            return false;
        }
        return !Log.HasLoggedErrors;
    }

    /// <inheritdoc/>
    public void Cancel()
    {
        lock (cancellationGate) if (!completed) cancellation.Cancel();
    }
    private void ReportDiagnostic(JsonElement diagnostic) => Diagnostics.BuildDiagnostics.Report(Log, diagnostic);
    private void Error(string code, string message) => Log.LogError(null, code, null, null, 0, 0, 0, 0, message);
}
