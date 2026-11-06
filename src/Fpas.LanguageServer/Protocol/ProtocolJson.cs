using Newtonsoft.Json.Linq;
using OmniSharp.Extensions.LanguageServer.Protocol.Serialization;

namespace Fpas.LanguageServer.Protocol;

internal static class ProtocolJson
{
    // Each conversion owns its serializer; the library supplies LSP union/URI converters.
    internal static JObject Object(object value) => JObject.FromObject(value, new LspSerializer().JsonSerializer);
    internal static JToken Token(object value) => JToken.FromObject(value, new LspSerializer().JsonSerializer);
}
