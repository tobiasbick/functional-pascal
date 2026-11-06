using System.Text;
using System.Text.Json;
using Fpas.Compiler;
using Fpas.Cli.Diagnostics;

namespace Fpas.Cli.Compilation;

internal static class CompilerCommand
{
    internal static CompileCommand Create()
    {
        var compile = new CompileCommand();
        compile.Command.SetAction(result => Execute(compile.Read(result)));
        return compile;
    }

    private static int Execute(CompileArguments options)
    {
        string? affectedPath = null;
        var sources = new List<CompilationSource>();
        try
        {
            foreach (var path in options.Sources)
            {
                affectedPath = path == "-" ? "<stdin>" : Path.GetFullPath(path);
                var text = path == "-" ? ReadStandardInput()
                    : new UTF8Encoding(false, true).GetString(File.ReadAllBytes(affectedPath));
                sources.Add(new(affectedPath, text));
            }
            var references = options.References.ToList();
            var core = Fpas.Deployment.DistributionAssets.Managed(typeof(Core.RecordAttribute).Assembly);
            if (!references.Any(path => Path.GetFileName(path).Equals("Fpas.Core.dll", StringComparison.OrdinalIgnoreCase))) references.Add(core);
            var result = FpasCompiler.Compile(new(sources,
                Path.GetFileNameWithoutExtension(options.Output), references, options.Target, options.EmitDebug, options.Documentation is not null, options.DiagnosticOptions, options.Optimize,
                StrongNameKey: options.KeyFile is null ? null : File.ReadAllBytes(options.KeyFile), PublicSign: options.PublicSign, DelaySign: options.DelaySign));
            if (!result.Success)
            {
                Report(options.Json, false, null, result.Diagnostics, sources);
                return 1;
            }
            affectedPath = options.Output;
            OutputFiles.Publish(options.Output, result.Program!, options.Documentation);
            Report(options.Json, true, options.Output, result.Diagnostics, sources);
            return 0;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or DecoderFallbackException)
        {
            Report(options.Json, false, null, [new("FP0002", exception.Message, null) { InputPath = affectedPath }]);
            return 1;
        }
    }

    private static string ReadStandardInput()
    {
        using var reader = new StreamReader(Console.OpenStandardInput(), new UTF8Encoding(false, true), detectEncodingFromByteOrderMarks: false);
        return reader.ReadToEnd();
    }

    internal static void Report(bool json, bool success, string? output, IReadOnlyList<CompilerDiagnostic> diagnostics, IReadOnlyList<CompilationSource>? sources = null)
    {
        if (json)
        {
            Console.WriteLine(JsonSerializer.Serialize(new(success, output, diagnostics), CompileJson.Default.CompileReport));
            return;
        }
        DiagnosticRenderer.Write(diagnostics, sources);
        if (success)
            Console.WriteLine($"Wrote {output}");
    }
}
