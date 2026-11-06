using System.Security.Cryptography;

namespace Fpas.Build.Tasks.Incremental;

internal static class SigningInputs
{
    internal static string Identity(string key, bool publicSign, bool delaySign)
    {
        if (key.Length == 0) return "";
        using var file = File.OpenRead(key);
        return "\0" + Path.GetFullPath(key) + "\0" + publicSign + "\0" + delaySign + "\0" + Convert.ToHexString(SHA256.HashData(file));
    }
}
