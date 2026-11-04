using ModelContextProtocol;

namespace Fpas.Debugging.Protocol;

internal sealed class DebugFailure(string code, string message) : McpException(code + ": " + message)
{
    internal string Code { get; } = code;
}
