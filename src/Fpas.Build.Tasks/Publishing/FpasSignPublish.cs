using Microsoft.Build.Framework;

namespace Fpas.Build.Tasks.Publishing;

public sealed class FpasSignPublish : Microsoft.Build.Utilities.Task
{
    [Required] public string Directory { get; set; } = "";
    [Required] public string Certificate { get; set; } = "";
    public string PasswordEnvironment { get; set; } = "";
    public bool Verify { get; set; }

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
