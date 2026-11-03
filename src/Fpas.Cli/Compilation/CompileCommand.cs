using System.CommandLine;
using System.CommandLine.Parsing;
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
    private readonly Option<bool> generateDocumentation = new("--generate-documentation") { Description = "Write XML documentation beside the DLL." };
    private readonly Option<string> documentation = new("--documentation") { Description = "Write XML documentation to this path (.xml)." };
    private readonly Option<bool> warnAsError = new("--warn-as-error") { Description = "Treat warnings as errors." };
    private readonly Option<string[]> warningsAsErrors = Repeated("--warnings-as-errors", "Promote warning codes (comma/semicolon separated); repeatable.");
    private readonly Option<string[]> warningsNotAsErrors = Repeated("--warnings-not-as-errors", "Exclude warning codes from --warn-as-error; repeatable.");
    private readonly Option<string[]> noWarn = Repeated("--no-warn", "Suppress warning codes (comma/semicolon separated); repeatable.");
    private readonly Option<string> diagnostics = new("--diagnostics") { Description = "Write one JSON result to stdout." };

    internal Option<string> ResponseFile { get; } = new("--response-file") { Description = "Read all compile arguments from a JSON string array; no inline arguments or nesting." };
    internal Command Command { get; } = new("compile", """
        Compile FPAS programs and units to a managed DLL.

        Requires net10.0 references, including System.Runtime.
        Executables include a runtimeconfig.json file.
        Exit codes: 0 success/help, 1 compilation or I/O failure, 2 invalid arguments.

        Examples:
          dotnet Fpas.Cli.dll compile examples/hello/main.fpas --output artifacts/Hello.dll --reference-dir <net10.0-reference-directory>
          dotnet Fpas.Cli.dll compile - --output artifacts/Hello.dll --reference-dir <net10.0-reference-directory> --diagnostics json
          dotnet Fpas.Cli.dll compile Library.fpas --target library --output artifacts/Library.dll --reference-dir <net10.0-reference-directory> --generate-documentation
        """);

    internal CompileCommand()
    {
        diagnostics.AcceptOnlyFromAmong("json");
        Command.Arguments.Add(sources);
        foreach (var option in new Option[] { output, references, referenceDirectories, target, debug, generateDocumentation,
            documentation, warnAsError, warningsAsErrors, warningsNotAsErrors, noWarn, diagnostics, ResponseFile })
            Command.Options.Add(option);
        Command.Validators.Add(result =>
        {
            if (result.GetValue(references)!.Length == 0 && result.GetValue(referenceDirectories)!.Length == 0)
                result.AddError("compile requires --reference or --reference-dir.");
        });
    }

    internal bool JsonRequested(ParseResult result) => result.GetResult(diagnostics)?.Tokens.Any(token => token.Value == "json") == true;

    internal void Validate(ParseResult result)
    {
        var sourceTokens = result.GetResult(sources)?.Tokens;
        foreach (var token in result.Tokens.TakeWhile(token => token.Type != TokenType.DoubleDash))
            if (sourceTokens?.Any(source => ReferenceEquals(source, token)) == true && token.Value.StartsWith('-') && token.Value != "-")
                throw new ArgumentException($"Unknown option '{token.Value}'. Use '--' before source paths starting with '-'.");
        foreach (var option in Command.Options.Where(option => option.ValueType != typeof(bool)))
            if (result.GetResult(option) is { } parsed && parsed.Tokens.Count < parsed.IdentifierTokenCount)
                throw new ArgumentException($"Each occurrence of '{option.Name}' requires a value.");
    }

    internal CompileArguments Read(ParseResult result)
    {
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
                Codes(result.GetValue(warningsNotAsErrors)!), Codes(result.GetValue(noWarn)!))).Validate();
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
