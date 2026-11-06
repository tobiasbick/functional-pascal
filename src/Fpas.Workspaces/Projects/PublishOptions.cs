using System.Runtime.InteropServices;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Projects;

public sealed record PublishOptions(string Mode = "aot", string? Runtime = null, string Configuration = "Release",
    bool NoRestore = false, bool NoBuild = false, string? KeyFile = null, bool PublicSign = false, bool DelaySign = false,
    string? Certificate = null, string? CertificatePasswordEnvironment = null)
{
    public string RuntimeIdentifier => Runtime ?? RuntimeInformation.RuntimeIdentifier;

    public void Validate()
    {
        if (Mode is not ("aot" or "single-file" or "self-contained" or "apphost" or "framework-dependent"))
            throw new ArgumentException("--mode must be aot, single-file, self-contained, apphost or framework-dependent.");
        if (RuntimeIdentifier is not ("win-x64" or "linux-x64")) throw new ArgumentException("Publishing supports win-x64 and linux-x64.");
        if (Mode == "aot" && RuntimeIdentifier != RuntimeInformation.RuntimeIdentifier)
            throw new ArgumentException("Native AOT must be built on the target operating system. Use Windows or native WSL/Linux for that target.");
        if (string.IsNullOrWhiteSpace(Configuration)) throw new ArgumentException("Configuration must not be empty.");
        if (PublicSign && DelaySign || KeyFile is null && (PublicSign || DelaySign))
            throw new ArgumentException("--key-file is required for signing; select at most one of --public-sign or --delay-sign.");
        if (KeyFile is not null && !File.Exists(KeyFile)) throw new ArgumentException("The signing key file does not exist.");
        if (NoBuild && KeyFile is not null) throw new ArgumentException("--no-build cannot change signing; configure it on the project before building.");
        if (Certificate is not null && !File.Exists(Certificate)) throw new ArgumentException("The publishing certificate file does not exist.");
        if (CertificatePasswordEnvironment is not null && (Certificate is null || !System.Text.RegularExpressions.Regex.IsMatch(CertificatePasswordEnvironment, "^[A-Za-z_][A-Za-z0-9_]*$")))
            throw new ArgumentException("--certificate-password-env requires --certificate and an environment variable name.");
    }

    internal IReadOnlyList<string> Arguments(ProjectOutput project, string output)
    {
        Validate();
        var args = new List<string> { "publish", project.Project, "--configuration", Processes.MsBuildSyntax.Escape(project.Configuration),
            "--runtime", RuntimeIdentifier, "--output", output, "--disable-build-servers", "-nr:false", "-p:UseSharedCompilation=false",
            "-p:Platform=" + Processes.MsBuildSyntax.Escape(project.Platform), "--verbosity", "minimal",
            "-p:PublishAot=" + (Mode == "aot"), "-p:PublishSingleFile=" + (Mode == "single-file"),
            "-p:SelfContained=" + (Mode is "aot" or "single-file" or "self-contained"),
            "-p:UseAppHost=" + (Mode != "framework-dependent"), "-p:PublishTrimmed=" + (Mode == "aot") };
        if (Mode == "single-file") args.Add("-p:IncludeNativeLibrariesForSelfExtract=true");
        if (Mode == "aot") args.AddRange(["-p:IlcTreatWarningsAsErrors=true", "-p:ILLinkTreatWarningsAsErrors=true"]);
        if (NoRestore) args.Add("--no-restore");
        if (NoBuild) args.Add("--no-build");
        if (KeyFile is not null) args.AddRange(["-p:SignAssembly=true", "-p:AssemblyOriginatorKeyFile=" + Processes.MsBuildSyntax.Escape(Path.GetFullPath(KeyFile)),
            "-p:PublicSign=" + PublicSign, "-p:DelaySign=" + DelaySign]);
        if (Certificate is not null) args.Add("-p:FpasPublishCertificate=" + Processes.MsBuildSyntax.Escape(Path.GetFullPath(Certificate)));
        if (CertificatePasswordEnvironment is not null) args.Add("-p:FpasPublishCertificatePasswordEnvironment=" + CertificatePasswordEnvironment);
        return args;
    }
}
