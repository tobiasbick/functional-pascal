using StreamJsonRpc;
using OmniSharp.Extensions.LanguageServer.Protocol;

namespace Fpas.LanguageServer.Protocol;

internal static class FileUris
{
    internal static string Path(Uri? uri)
    {
        if (uri is null || !uri.IsAbsoluteUri || !uri.IsFile || uri.IsUnc || uri.Host.Length != 0 && uri.Host != "localhost" || uri.Query.Length != 0 || uri.Fragment.Length != 0)
            throw new LocalRpcException("Supply a local file URI.") { ErrorCode = -32602 };
        var local = uri.LocalPath;
        // Uri keeps the root slash when a Windows drive separator is percent-encoded.
        if (OperatingSystem.IsWindows() && local.Length >= 4 && local[0] == '/' &&
            char.IsAsciiLetter(local[1]) && local[2] == ':' && local[3] == '/')
            local = local[1..];
        if (!System.IO.Path.IsPathFullyQualified(local))
            throw new LocalRpcException("Supply an absolute local file URI.") { ErrorCode = -32602 };
        return System.IO.Path.GetFullPath(local);
    }
    internal static string Context(string value)
    {
        try
        {
            if (System.IO.Path.IsPathFullyQualified(value) && !value.StartsWith("\\\\", StringComparison.Ordinal)) return System.IO.Path.GetFullPath(value);
            if (System.Uri.TryCreate(value, UriKind.Absolute, out var uri)) return Path(uri);
        }
        catch (Exception error) when (error is ArgumentException or NotSupportedException or PathTooLongException)
        { throw new LocalRpcException("Supply a valid project/solution path.") { ErrorCode = -32602 }; }
        throw new LocalRpcException("initializationOptions.context requires an absolute project/solution path or local file URI.") { ErrorCode = -32602 };
    }
    internal static DocumentUri Uri(string path) => DocumentUri.FromFileSystemPath(System.IO.Path.GetFullPath(path));
}
