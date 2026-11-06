using System.Buffers;
using System.Text;
using Newtonsoft.Json;
using StreamJsonRpc;
using StreamJsonRpc.Protocol;

namespace Fpas.LanguageServer.Transport;

/// <summary>Uses the library's JSON codec with explicit message and serialization budgets.</summary>
internal sealed class BoundedFormatter : IJsonRpcMessageTextFormatter, IJsonRpcInstanceContainer, IJsonRpcMessageFactory, IDisposable
{
    internal const int InputLimit = 8_388_608;
    internal const int OutputLimit = 4_194_304;
    private readonly JsonMessageFormatter inner = new();
    internal JsonSerializer Serializer => inner.JsonSerializer;
    public JsonRpc Rpc { set => ((IJsonRpcInstanceContainer)inner).Rpc = value; }
    public Encoding Encoding { get => inner.Encoding; set => inner.Encoding = value; }
    internal BoundedFormatter()
    {
        inner.Encoding = new UTF8Encoding(false, true); inner.JsonSerializer.MaxDepth = 64;
        inner.JsonSerializer.NullValueHandling = NullValueHandling.Ignore;
        inner.JsonSerializer.TypeNameHandling = TypeNameHandling.None;
        inner.JsonSerializer.DateParseHandling = DateParseHandling.None;
    }
    public JsonRpcMessage Deserialize(ReadOnlySequence<byte> bytes) => Deserialize(bytes, Encoding);
    public JsonRpcMessage Deserialize(ReadOnlySequence<byte> bytes, Encoding encoding)
    {
        if (bytes.Length > InputLimit) throw new InvalidDataException("LSP message exceeds 8 MiB.");
        if (encoding.CodePage != System.Text.Encoding.UTF8.CodePage) throw new InvalidDataException("LSP requires UTF-8 messages.");
        return inner.Deserialize(bytes, new UTF8Encoding(false, true));
    }
    public void Serialize(IBufferWriter<byte> writer, JsonRpcMessage message) => inner.Serialize(new LimitedWriter(writer), message);
    public object GetJsonText(JsonRpcMessage message) => inner.GetJsonText(message);
    public JsonRpcRequest CreateRequestMessage() => ((IJsonRpcMessageFactory)inner).CreateRequestMessage();
    public JsonRpcError CreateErrorMessage() => ((IJsonRpcMessageFactory)inner).CreateErrorMessage();
    public JsonRpcResult CreateResultMessage() => ((IJsonRpcMessageFactory)inner).CreateResultMessage();
    public void Dispose() => inner.Dispose();

    private sealed class LimitedWriter(IBufferWriter<byte> inner) : IBufferWriter<byte>
    {
        private int written;
        public void Advance(int count)
        { if (count < 0 || count > OutputLimit - written) throw new InvalidDataException("LSP response exceeds 4 MiB."); inner.Advance(count); written += count; }
        public Memory<byte> GetMemory(int sizeHint = 0)
        {
            if (sizeHint < 0 || Math.Max(1, sizeHint) > OutputLimit - written) throw new InvalidDataException("LSP response exceeds 4 MiB.");
            var memory = inner.GetMemory(sizeHint); return memory[..Math.Min(memory.Length, OutputLimit - written)];
        }
        public Span<byte> GetSpan(int sizeHint = 0) => GetMemory(sizeHint).Span;
    }
}
