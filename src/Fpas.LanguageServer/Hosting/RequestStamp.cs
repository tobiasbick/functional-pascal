// [Documentation](../../../docs/architecture/workspaces.md#language-server-adapter)
namespace Fpas.LanguageServer.Hosting;

internal sealed record RequestStamp(long Session, long Workspace);
