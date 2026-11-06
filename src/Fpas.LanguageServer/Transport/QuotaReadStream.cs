using System.Globalization;
using System.Text;

namespace Fpas.LanguageServer.Transport;

/// <summary>Inspects quotas before forwarding unchanged bytes to the library's framing reader.</summary>
internal sealed class QuotaReadStream(Stream inner) : Stream
{
    private readonly byte[] header = new byte[8192];
    private int headerLength;
    private int remaining;

    private void Inspect(ReadOnlySpan<byte> bytes)
    {
        while (!bytes.IsEmpty)
        {
            if (remaining != 0)
            {
                var count = Math.Min(remaining, bytes.Length); remaining -= count; bytes = bytes[count..]; continue;
            }
            var value = bytes[0]; bytes = bytes[1..];
            if (headerLength == header.Length || value > 127) throw new InvalidDataException("Invalid or oversized LSP header.");
            header[headerLength++] = value;
            if (headerLength >= 4 && header.AsSpan(headerLength - 4, 4).SequenceEqual("\r\n\r\n"u8))
            {
                int? length = null;
                foreach (var line in Encoding.ASCII.GetString(header, 0, headerLength - 4).Split("\r\n"))
                {
                    var colon = line.IndexOf(':');
                    if (colon < 1) throw new InvalidDataException("Invalid LSP header field.");
                    if (!line[..colon].Equals("Content-Length", StringComparison.OrdinalIgnoreCase)) continue;
                    if (length is not null || !int.TryParse(line[(colon + 1)..].Trim(), NumberStyles.None, CultureInfo.InvariantCulture, out var size) || size is < 1 or > BoundedFormatter.InputLimit)
                        throw new InvalidDataException("LSP Content-Length must be 1..8 MiB and occur once.");
                    length = size;
                }
                remaining = length ?? throw new InvalidDataException("Missing LSP Content-Length."); headerLength = 0;
            }
        }
    }
    public override async ValueTask<int> ReadAsync(Memory<byte> buffer, CancellationToken token = default)
    { var count = await inner.ReadAsync(buffer, token); Inspect(buffer.Span[..count]); return count; }
    public override int Read(byte[] buffer, int offset, int count) { var actual = inner.Read(buffer, offset, count); Inspect(buffer.AsSpan(offset, actual)); return actual; }
    public override Task<int> ReadAsync(byte[] buffer, int offset, int count, CancellationToken token) => ReadAsync(buffer.AsMemory(offset, count), token).AsTask();
    public override bool CanRead => true;
    public override bool CanSeek => false;
    public override bool CanWrite => false;
    public override long Length => throw new NotSupportedException();
    public override long Position { get => throw new NotSupportedException(); set => throw new NotSupportedException(); }
    public override void Flush() { }
    public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
    public override void SetLength(long value) => throw new NotSupportedException();
    public override void Write(byte[] buffer, int offset, int count) => throw new NotSupportedException();
}
