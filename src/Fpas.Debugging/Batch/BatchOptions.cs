using System.Globalization;
using Fpas.Debugging.Core;
using Fpas.Debugging.Evaluation;

namespace Fpas.Debugging.Batch;

public sealed record BatchOptions(string? Program, string? Cwd = null, string[]? Breakpoints = null, string[]? Expressions = null,
    string? Stdin = null, int Timeout = 30, int OutputLimit = 1_048_576, string[]? Arguments = null);

internal sealed record BatchRequest(string Program, string Cwd, SourceBreakpoint[] Breakpoints, string[] Expressions,
    string? Stdin, int Timeout, int OutputLimit, string[] Arguments)
{
    internal static BatchRequest Create(BatchOptions options)
    {
        if (options.Timeout is < 1 or > 3600 || options.OutputLimit is < 1 or > 1_048_576)
            throw new ArgumentException("Use --timeout 1..3600 and --output-limit 1..1048576.");
        if (string.IsNullOrWhiteSpace(options.Program)) throw new ArgumentException("A built executable DLL is required. See fpas debug --help.");
        var program = Path.GetFullPath(options.Program);
        if (!program.EndsWith(".dll", StringComparison.OrdinalIgnoreCase) || !File.Exists(program))
            throw new IOException("Build the target first and supply its existing executable DLL.");
        if (!File.Exists(Path.ChangeExtension(program, ".runtimeconfig.json")))
            throw new IOException("A runtimeconfig.json is required beside the executable DLL.");
        var cwd = options.Cwd is null ? Path.GetDirectoryName(program)! : Path.GetFullPath(options.Cwd);
        if (!Directory.Exists(cwd)) throw new IOException("The target working directory does not exist.");
        var expressions = options.Expressions ?? [];
        if ((options.Breakpoints?.Length ?? 0) > 64 || expressions.Length > 64)
            throw new ArgumentException("Supply at most 64 breakpoints and 64 expressions.");
        foreach (var expression in expressions)
        {
            try { _ = ExpressionParser.Parse(expression); }
            catch (DebugException failure) { throw new ArgumentException(failure.Message, failure); }
        }
        var comparison = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        var points = (options.Breakpoints ?? []).Select(text => ParseBreakpoint(text, cwd))
            .DistinctBy(point => (point.Source, point.Line, point.Column), new BreakpointComparer(comparison)).ToArray();
        var arguments = options.Arguments ?? [];
        if (arguments.Length > 1024 || arguments.Any(item => item is null || item.Contains('\0')) || arguments.Sum(item => (long)item.Length) > 65536)
            throw new ArgumentException("Target arguments require at most 1024 elements and 65536 UTF-16 code units, without NUL.");
        return new(program, cwd, points, expressions, options.Stdin is null ? null : Path.GetFullPath(options.Stdin, cwd), options.Timeout, options.OutputLimit, arguments);
    }

    private static SourceBreakpoint ParseBreakpoint(string text, string cwd)
    {
        var last = text.LastIndexOf(':');
        if (last <= 0 || !Positive(text[(last + 1)..], out var value)) throw new ArgumentException("Use --break path:line[:column] with positive coordinates.");
        var path = text[..last]; int? column = null; var line = value;
        var previous = path.LastIndexOf(':');
        if (previous > 0 && Positive(path[(previous + 1)..], out var earlier)) { column = value; line = earlier; path = path[..previous]; }
        if (string.IsNullOrWhiteSpace(path)) throw new ArgumentException("A breakpoint needs a source path.");
        return new(Path.GetFullPath(path, cwd), line, column);
    }

    private static bool Positive(string text, out int number) => int.TryParse(text, NumberStyles.None, CultureInfo.InvariantCulture, out number) && number > 0;

    private sealed class BreakpointComparer(StringComparer paths) : IEqualityComparer<(string Source, int Line, int? Column)>
    {
        public bool Equals((string Source, int Line, int? Column) x, (string Source, int Line, int? Column) y)
            => paths.Equals(x.Source, y.Source) && x.Line == y.Line && x.Column == y.Column;
        public int GetHashCode((string Source, int Line, int? Column) value) => HashCode.Combine(paths.GetHashCode(value.Source), value.Line, value.Column);
    }
}
