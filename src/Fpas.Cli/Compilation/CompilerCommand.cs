using System.CommandLine;
using System.CommandLine.Help;
using System.Text;
using System.Text.Json;
using Fpas.Compiler;

namespace Fpas.Cli.Compilation;

internal static class CompilerCommand
{
    private static readonly ParserConfiguration ParserConfiguration = new() { ResponseFileTokenReplacer = null };
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        Converters = { new System.Text.Json.Serialization.JsonStringEnumConverter(JsonNamingPolicy.CamelCase) }
    };

    internal static int Run(string[] args)
    {
        Console.OutputEncoding = new UTF8Encoding(false);
        var compile = new CompileCommand();
        compile.Command.SetAction(result => Execute(compile.Read(result)));
        var root = new Command("fpas", """
            FPAS compiler for .NET

            Examples:
              fpas compile --help
              fpas compile examples/hello/main.fpas --output artifacts/Hello.dll --reference-dir <net10.0-reference-directory>
            """);
        root.Options.Add(new HelpOption());
        root.Options.Add(new VersionOption());
        root.Subcommands.Add(compile.Command);
        var jsonRequested = false;
        try
        {
            var parsed = root.Parse(args.Length == 0 ? ["--help"] : args, ParserConfiguration);
            jsonRequested = compile.JsonRequested(parsed);
            var expanded = CompileResponseFile.Expand(parsed, compile, ParserConfiguration);
            var fromResponseFile = expanded != parsed;
            parsed = expanded;
            jsonRequested |= compile.JsonRequested(parsed);
            if (fromResponseFile && parsed.GetResult(compile.ResponseFile) is not null)
                throw new ArgumentException("Response file nesting is unsupported.");
            compile.Validate(parsed);
            if (parsed.Errors.Count > 0)
                throw new ArgumentException(string.Join(" ", parsed.Errors.Select(error => error.Message)));
            return parsed.Invoke(new InvocationConfiguration { EnableDefaultExceptionHandler = false });
        }
        catch (ArgumentException exception) when (exception is not DecoderFallbackException)
        {
            Report(jsonRequested, false, null, [new("FP0001", exception.Message + " See fpas compile --help.", new("<command line>", 1, 1, 0))]);
            return 2;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException)
        {
            Report(jsonRequested, false, null, [new("FP0002", exception.Message, new("<command line>", 1, 1, 0))]);
            return 1;
        }
    }

    private static int Execute(CompileArguments options)
    {
        var inputPath = "<command line>";
        try
        {
            var sources = new List<CompilationSource>();
            foreach (var path in options.Sources)
            {
                inputPath = path == "-" ? "<stdin>" : Path.GetFullPath(path);
                var text = path == "-" ? ReadStandardInput()
                    : new UTF8Encoding(false, true).GetString(File.ReadAllBytes(inputPath));
                sources.Add(new(inputPath, text));
            }
            var references = options.References.ToList();
            var core = typeof(Core.RecordAttribute).Assembly.Location;
            if (!references.Any(path => Path.GetFileName(path).Equals("Fpas.Core.dll", StringComparison.OrdinalIgnoreCase))) references.Add(core);
            var result = FpasCompiler.Compile(new(sources,
                Path.GetFileNameWithoutExtension(options.Output), references, options.Target, options.EmitDebug, options.Documentation is not null, options.DiagnosticOptions, options.Optimize));
            if (!result.Success)
            {
                Report(options.Json, false, null, result.Diagnostics);
                return 1;
            }
            OutputFiles.Publish(options.Output, result.Program!, options.Documentation);
            Report(options.Json, true, options.Output, result.Diagnostics);
            return 0;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException)
        {
            Report(options.Json, false, null, [new("FP0002", exception.Message, new(inputPath, 1, 1, 0))]);
            return 1;
        }
    }

    private static string ReadStandardInput()
    {
        using var reader = new StreamReader(Console.OpenStandardInput(), new UTF8Encoding(false, true), detectEncodingFromByteOrderMarks: false);
        return reader.ReadToEnd();
    }

    private static void Report(bool json, bool success, string? output, IReadOnlyList<CompilerDiagnostic> diagnostics)
    {
        if (json)
        {
            Console.WriteLine(JsonSerializer.Serialize(new { success, output, diagnostics }, JsonOptions));
            return;
        }
        foreach (var diagnostic in diagnostics)
            Console.Error.WriteLine($"{diagnostic.Location.Path}({diagnostic.Location.Line},{diagnostic.Location.Column}): {(diagnostic.Severity == DiagnosticSeverity.Warning ? "warning" : "error")} {diagnostic.Code}: {diagnostic.Message}");
        if (success)
            Console.WriteLine($"Wrote {output}");
    }
}
