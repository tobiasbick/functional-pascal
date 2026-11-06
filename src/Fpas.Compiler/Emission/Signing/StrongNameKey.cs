using System.Buffers.Binary;
using System.Reflection.Metadata;
using System.Security.Cryptography;

namespace Fpas.Compiler.Emission.Signing;

internal sealed class StrongNameKey : IDisposable
{
    private readonly RSA rsa;
    internal byte[] PublicKey { get; }
    internal int SignatureSize => rsa.KeySize / 8;

    internal StrongNameKey(byte[] key, bool publicOnly)
    {
        rsa = RSA.Create();
        try
        {
            var blob = key.AsSpan();
            if (blob.Length >= 32 && BinaryPrimitives.ReadInt32LittleEndian(blob[8..]) == blob.Length - 12) blob = blob[12..];
            if (blob.Length < 20 || blob[1] != 2 || blob[0] is not (6 or 7) ||
                BinaryPrimitives.ReadUInt32LittleEndian(blob[8..]) != (blob[0] == 7 ? 0x32415352u : 0x31415352u))
                throw new CryptographicException("Use a CAPI RSA .snk key pair or strong-name public key.");
            var bits = BinaryPrimitives.ReadUInt32LittleEndian(blob[12..]);
            if (bits < 1024 || bits > 16384 || bits % 16 != 0) throw new CryptographicException("Strong-name RSA keys must have 1024..16384 bits.");
            var size = (int)bits / 8;
            var privateKey = blob[0] == 7;
            if (blob.Length != 20 + (privateKey ? size * 9 / 2 : size)) throw new CryptographicException("Invalid strong-name key length.");
            var offset = 20; var data = blob.ToArray();
            byte[] Read(int count) { var value = data.AsSpan(offset, count).ToArray(); offset += count; Array.Reverse(value); return value; }
            var exponent = new byte[4]; BinaryPrimitives.WriteUInt32BigEndian(exponent, BinaryPrimitives.ReadUInt32LittleEndian(blob[16..]));
            var parameters = new RSAParameters { Modulus = Read(size), Exponent = exponent.SkipWhile(value => value == 0).ToArray() };
            if (privateKey)
            {
                parameters.P = Read(size / 2); parameters.Q = Read(size / 2); parameters.DP = Read(size / 2);
                parameters.DQ = Read(size / 2); parameters.InverseQ = Read(size / 2); parameters.D = Read(size);
            }
            if (!privateKey && !publicOnly) throw new CryptographicException("Full signing requires a private .snk key pair.");
            rsa.ImportParameters(parameters);
            PublicKey = new byte[32 + size];
            BinaryPrimitives.WriteUInt32LittleEndian(PublicKey, 0x2400);
            BinaryPrimitives.WriteUInt32LittleEndian(PublicKey.AsSpan(4), 0x8004);
            BinaryPrimitives.WriteInt32LittleEndian(PublicKey.AsSpan(8), 20 + size);
            blob[..(20 + size)].CopyTo(PublicKey.AsSpan(12));
            BinaryPrimitives.WriteUInt32LittleEndian(PublicKey.AsSpan(16), 0x2400);
            PublicKey[12] = 6; BinaryPrimitives.WriteUInt32LittleEndian(PublicKey.AsSpan(20), 0x31415352);
        }
        catch { rsa.Dispose(); throw; }
    }

    internal byte[] Sign(IEnumerable<Blob> content)
    {
        using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA1);
        foreach (var blob in content) hash.AppendData(blob.GetBytes());
        var signature = rsa.SignHash(hash.GetHashAndReset(), HashAlgorithmName.SHA1, RSASignaturePadding.Pkcs1);
        Array.Reverse(signature); return signature;
    }

    public void Dispose() => rsa.Dispose();
}
