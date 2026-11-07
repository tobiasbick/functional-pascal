using Newtonsoft.Json.Linq;
using Newtonsoft.Json;
using OmniSharp.Extensions.LanguageServer.Protocol.Serialization;

namespace Fpas.LanguageServer.Protocol;

internal static class ProtocolJson
{
    // Each conversion owns its serializer; the library supplies LSP union/URI converters.
    internal static JObject Object(object value) => JObject.FromObject(value, new LspSerializer().JsonSerializer);
    internal static JToken Token(object value, bool omitNulls = false)
    {
        var serializer = new LspSerializer().JsonSerializer;
        if (omitNulls) serializer.NullValueHandling = NullValueHandling.Ignore;
        return JToken.FromObject(value, serializer);
    }
}
