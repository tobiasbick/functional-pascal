using System.Text;
using System.Text.Json;
using Fpas.Compiler;

namespace Fpas.Cli.Compilation;

internal static class CompilerCommand
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        Converters = { new System.Text.Json.Serialization.JsonStringEnumConverter(JsonNamingPolicy.CamelCase) }
    };

    internal static CompileCommand Create()
    {
        var compile = new CompileCommand();
        compile.Command.SetAction(result => Execute(compile.Read(result)));
        return compile;
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

    internal static void Report(bool json, bool success, string? output, IReadOnlyList<CompilerDiagnostic> diagnostics)
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
