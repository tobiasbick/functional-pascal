using Microsoft.Build.Framework;

namespace Fpas.Build.Tasks.Publishing;

/// <summary>Signs or verifies a publish directory with <see cref="Fpas.Deployment.PublishSignature"/>.</summary>
public sealed class FpasSignPublish : Microsoft.Build.Utilities.Task
{
    /// <summary>Gets or sets the publish directory.</summary>
    [Required] public string Directory { get; set; } = "";
    /// <summary>Gets or sets the signing (PKCS#12) or pinned verification certificate.</summary>
    [Required] public string Certificate { get; set; } = "";
    /// <summary>Gets or sets the environment variable holding the certificate password.</summary>
    public string PasswordEnvironment { get; set; } = "";
    /// <summary>Gets or sets whether to verify instead of sign.</summary>
    public bool Verify { get; set; }

    /// <inheritdoc/>
    public override bool Execute()
    {
        try
        {
            if (Verify) Fpas.Deployment.PublishSignature.Verify(Directory, Certificate);
            else Fpas.Deployment.PublishSignature.Sign(Directory, Certificate, PasswordEnvironment);
            return true;
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or System.Security.Cryptography.CryptographicException or
            ArgumentException or System.Text.Json.JsonException)
        {
            Log.LogError(null, "FP5011", null, null, 0, 0, 0, 0, Verify
                ? "Publish signature verification failed. Check the pinned certificate and published file contents."
                : "Publish signing failed. Check the certificate, private key and password environment variable.");
            return false;
        }
    }
}
