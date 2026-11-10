using System.CommandLine;
using Fpas.Compiler;

namespace Fpas.Cli.Compilation;

internal sealed class CompileCommand
{
    private readonly Argument<string[]> sources = new("sources")
    {
        Description = "FPAS source paths; '-' reads one source from strict UTF-8 stdin.",
        Arity = ArgumentArity.OneOrMore
    };
    private readonly Option<string> output = new("--output") { Description = "Managed assembly output (.dll).", Required = true };
    private readonly Option<string[]> references = Repeated("--reference", "Supply an assembly reference; repeatable.");
    private readonly Option<string[]> referenceDirectories = Repeated("--reference-dir", "Supply all DLLs in a directory; repeatable.");
    private readonly Option<string> target = Choice("--target", "Output kind.", "exe", "library");
    private readonly Option<string> debug = Choice("--debug", "Debug symbol format.", "portable", "none");
    private readonly Option<bool> optimize = new("--optimize") { Description = "Allow JIT optimization while retaining the selected debug symbols." };
    private readonly Option<bool> generateDocumentation = new("--generate-documentation") { Description = "Write XML documentation beside the DLL." };
    private readonly Option<string> documentation = new("--documentation") { Description = "Write XML documentation to this path (.xml)." };
    private readonly Option<bool> warnAsError = new("--warn-as-error") { Description = "Treat warnings as errors." };
    private readonly Option<string[]> warningsAsErrors = Repeated("--warnings-as-errors", "Promote warning codes (comma/semicolon separated); repeatable.");
    private readonly Option<string[]> warningsNotAsErrors = Repeated("--warnings-not-as-errors", "Exclude warning codes from --warn-as-error; repeatable.");
    private readonly Option<string[]> noWarn = Repeated("--no-warn", "Suppress warning codes (comma/semicolon separated); repeatable.");
    private readonly Option<string> diagnostics = new("--diagnostics") { Description = "Write one JSON result to stdout." };
    private readonly Option<string> expectedCompiler = new("--expected-compiler-sha256") { Hidden = true };
    private readonly Option<string> keyFile = new("--key-file") { Description = "Strong-name RSA .snk key file." };
    private readonly Option<bool> publicSign = new("--public-sign") { Description = "Embed the public key without a cryptographic signature." };
    private readonly Option<bool> delaySign = new("--delay-sign") { Description = "Reserve a signature for later signing; requires --key-file." };

    internal Option<string> ResponseFile { get; } = new("--response-file") { Description = "Read all compile arguments from a JSON string array; no inline arguments or nesting." };
    internal Command Command { get; } = new("compile", """
        Compile FPAS programs and units to a managed DLL.

        Requires net10.0 references, including System.Runtime.
        Executables include a runtimeconfig.json file.
        Exit codes: 0 success/help, 1 compilation or I/O failure, 2 invalid arguments.

        Examples:
          fpas compile examples/basics/hello/main.fpas --output artifacts/Hello.dll --reference-dir <net10.0-reference-directory>

          fpas compile - --output artifacts/Hello.dll --reference-dir <net10.0-reference-directory> --diagnostics json

          fpas compile Library.fpas --target library --output artifacts/Library.dll --reference-dir <net10.0-reference-directory> --generate-documentation
        """);

    internal CompileCommand()
    {
        diagnostics.AcceptOnlyFromAmong("json");
        Command.Arguments.Add(sources);
        foreach (var option in new Option[] { output, references, referenceDirectories, target, debug, optimize, generateDocumentation,
            documentation, warnAsError, warningsAsErrors, warningsNotAsErrors, noWarn, diagnostics, expectedCompiler, keyFile, publicSign, delaySign, ResponseFile })
            Command.Options.Add(option);
        Command.Validators.Add(result =>
        {
            if (result.GetValue(references)!.Length == 0 && result.GetValue(referenceDirectories)!.Length == 0)
                result.AddError("compile requires --reference or --reference-dir.");
        });
    }

    internal bool JsonRequested(ParseResult result) => result.GetResult(diagnostics)?.Tokens.Any(token => token.Value == "json") == true;

    internal void Validate(ParseResult result)
        => CommandArguments.Validate(result, sources, standardInput: true);

    internal CompileArguments Read(ParseResult result)
    {
        if (result.GetValue(publicSign) && result.GetValue(delaySign) || result.GetValue(keyFile) is null && (result.GetValue(publicSign) || result.GetValue(delaySign)))
            throw new ArgumentException("Select --key-file and at most one of --public-sign or --delay-sign.");
        if (result.GetValue(expectedCompiler) is { } expected)
        {
            using var compiler = File.OpenRead(Fpas.Deployment.DistributionAssets.Managed(typeof(FpasCompiler).Assembly));
            if (Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(compiler)) != expected)
                throw new ArgumentException("Selected SDK compiler differs from this CLI; select the matching SDK or FpasCompilerPath.");
        }
        var referencePaths = result.GetValue(references)!.Select(Path.GetFullPath).ToList();
        foreach (var directory in result.GetValue(referenceDirectories)!)
            referencePaths.AddRange(Directory.EnumerateFiles(Path.GetFullPath(directory), "*.dll").Order(StringComparer.Ordinal));
        var outputPath = result.GetValue(output)!;
        var documentationPath = result.GetValue(documentation);
        if (documentationPath is null && result.GetValue(generateDocumentation))
            documentationPath = Path.ChangeExtension(outputPath, ".xml");
        return new CompileArguments(result.GetValue(sources)!, outputPath, referencePaths, JsonRequested(result),
            result.GetValue(target) == "library" ? CompilationTarget.Library : CompilationTarget.Exe,
            result.GetValue(debug) == "portable", documentationPath,
            new(result.GetValue(warnAsError), Codes(result.GetValue(warningsAsErrors)!),
                Codes(result.GetValue(warningsNotAsErrors)!), Codes(result.GetValue(noWarn)!)), result.GetValue(optimize),
                result.GetValue(keyFile), result.GetValue(publicSign), result.GetValue(delaySign)).Validate();
    }

    private static string[] Codes(string[] values) => values.SelectMany(value =>
        value.Split([';', ','], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)).ToArray();

    private static Option<string[]> Repeated(string name, string description) => new(name)
    {
        Description = description,
        Arity = ArgumentArity.OneOrMore,
        AllowMultipleArgumentsPerToken = false
    };

    private static Option<string> Choice(string name, string description, params string[] values)
    {
        var option = new Option<string>(name) { Description = description, DefaultValueFactory = _ => values[0] };
        option.AcceptOnlyFromAmong(values);
        return option;
    }
}
