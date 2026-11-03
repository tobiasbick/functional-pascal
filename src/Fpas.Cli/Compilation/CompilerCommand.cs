using System.Text;
using System.Text.Json;
using Fpas.Compiler;

namespace Fpas.Cli.Compilation;

internal static class CompilerCommand
{
    private const string RootHelp = """
        FPAS compiler for .NET

        Commands:
          compile    Compile FPAS programs and units to a managed DLL.

        Help: compile --help
        Example:
          dotnet Fpas.Cli.dll compile examples/hello/main.fpas --output artifacts/Hello.dll --reference-dir <net10.0-reference-directory>
        """;

    private const string CompileHelp = """
        Usage: compile <source.fpas|-> [more-sources.fpas...] --output <program.dll> [references] [options]

        References:
          --reference <assembly.dll>       Supply an assembly reference; repeatable.
          --reference-dir <directory>      Supply all DLLs in a reference directory; repeatable.
        Options:
          --target exe|library             Select program+units or units-only output; default exe.
          --debug portable|none            Emit a portable PDB or disable symbols; default portable.
          --generate-documentation         Write standard XML documentation beside the DLL.
          --documentation <file.xml>       Write XML documentation to this path instead.
          --warn-as-error                  Treat warnings as errors.
          --warnings-as-errors <codes>      Promote selected warning codes (comma/semicolon separated).
          --warnings-not-as-errors <codes>  Exclude selected codes from --warn-as-error.
          --no-warn <codes>                 Suppress selected warning codes.
          --response-file <file.json>       Read compile arguments from a JSON string array.
          --diagnostics json               Write one JSON result to stdout.
          --help                           Show this help.

        The initial compiler requires net10.0 references, including System.Runtime.
        '-' reads one source from stdin. Executables include a runtimeconfig.json file.
        Exit codes: 0 success/help, 1 compilation or I/O failure, 2 invalid arguments.

        Examples:
          dotnet Fpas.Cli.dll compile examples/hello/main.fpas --output artifacts/Hello.dll --reference-dir <net10.0-reference-directory>
          dotnet Fpas.Cli.dll compile - --output artifacts/Hello.dll --reference-dir <net10.0-reference-directory> --diagnostics json
          dotnet Fpas.Cli.dll compile Library.fpas --target library --output artifacts/Library.dll --reference-dir <net10.0-reference-directory> --generate-documentation
        """;

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        Converters = { new System.Text.Json.Serialization.JsonStringEnumConverter(JsonNamingPolicy.CamelCase) }
    };

    internal static int Run(string[] args)
    {
        Console.OutputEncoding = new UTF8Encoding(false);
        if (args.Length == 0 || args is ["--help"])
        {
            Console.WriteLine(RootHelp);
            return 0;
        }
        if (args is ["compile", "--help"])
        {
            Console.WriteLine(CompileHelp);
            return 0;
        }
        CompileArguments? options = null;
        var inputPath = "<command line>";
        var jsonRequested = args.Zip(args.Skip(1)).Any(pair => pair.First == "--diagnostics" && pair.Second == "json");
        try
        {
            if (args[0] != "compile")
                throw new ArgumentException($"Unknown command '{args[0]}'. Use compile --help.");
            var expanded = CompileArguments.Expand(args[1..]);
            jsonRequested |= expanded.Zip(expanded.Skip(1)).Any(pair => pair.First == "--diagnostics" && pair.Second == "json");
            options = CompileArguments.Parse(expanded);
            var sources = new List<CompilationSource>();
            foreach (var path in options.Sources)
            {
                inputPath = path == "-" ? "<stdin>" : path;
                var text = path == "-" ? ReadStandardInput()
                    : new UTF8Encoding(false, true).GetString(File.ReadAllBytes(path));
                sources.Add(new(inputPath, text));
            }
            var references = options.References.ToList();
            var core = typeof(Core.RecordAttribute).Assembly.Location;
            if (!references.Any(path => Path.GetFileName(path).Equals("Fpas.Core.dll", StringComparison.OrdinalIgnoreCase))) references.Add(core);
            var result = FpasCompiler.Compile(new(sources,
                Path.GetFileNameWithoutExtension(options.Output), references, options.Target, options.EmitDebug, options.Documentation is not null, options.DiagnosticOptions));
            if (!result.Success)
            {
                Report(options.Json, false, null, result.Diagnostics);
                return 1;
            }
            OutputFiles.Publish(options.Output, result.Program!, options.Documentation);
            Report(options.Json, true, options.Output, result.Diagnostics);
            return 0;
        }
        catch (ArgumentException exception) when (exception is not DecoderFallbackException)
        {
            Report(options?.Json ?? jsonRequested, false, null, [new("FP0001", exception.Message + " See compile --help.", new("<command line>", 1, 1, 0))]);
            return 2;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException)
        {
            Report(options?.Json ?? jsonRequested, false, null, [new("FP0002", exception.Message, new(inputPath, 1, 1, 0))]);
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
