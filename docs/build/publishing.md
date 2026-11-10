# Publish FPAS applications

`fpas publish` uses the .NET SDK to deliver an executable project, a selected
solution application or a standalone `.fpas` source. Native AOT is the default.
The CLI supports Windows x64 and Linux x64/glibc, including native Linux in WSL.
The normal FPAS/C#/F# project dependency graph and matching SDK selection apply.

```sh
fpas publish App/App.fpasproj --output artifacts/app --runtime win-x64
fpas publish App/App.fpasproj --output artifacts/app-debug --configuration Debug
fpas publish App.slnx --project App/App.fpasproj --output artifacts/app --mode single-file
fpas publish main.fpas --sdk-feed ./feed --output artifacts/app --report json
```

## Modes and prerequisites

| `--mode` | Delivered output |
| --- | --- |
| `aot` (default) | Native executable with compiled-in runtime; no installed .NET runtime needed |
| `single-file` | Self-contained managed bundle with native runtime extraction; no installed .NET runtime needed |
| `self-contained` | Apphost, runtime and dependency files; copy the whole directory |
| `apphost` | Launcher and managed dependencies; requires the target .NET runtime |
| `framework-dependent` | Managed DLL, runtimeconfig/deps and dependencies; run through dotnet |

`--configuration` defaults to Release; Debug is supported. `--runtime` defaults
to the host RID and accepts `win-x64` or `linux-x64`. Native AOT builds on the
target OS: use Windows for Windows and WSL/Linux for Linux. It needs the .NET 10
SDK and native build tools: Visual Studio's C++ desktop workload on Windows,
Clang and zlib development headers on Ubuntu (`clang zlib1g-dev`). Restore requires
the selected runtime/AOT packages; once cached,
`--no-restore` supports offline publication. FPAS CIL remains AnyCPU; the RID
selects the host/runtime and native compiler.

Core and foreign dependencies follow the SDK runtime asset graph. AOT/single-file
executables run independently of compiler, sources and feed. PDBs are optional
for execution; retain them for debugging. Project-requested content files can
remain separate and must also be copied. Directory modes need all runtime files.

For applications using the [packaged CUI binding](../../src/Fpas.Gui.Cui/README.md#packaged-binding),
directory modes include the shared CUI library beside the application. Single-file
publication bundles it for native extraction. Use the binding package for
Native AOT so its transitive targets link CUI statically.
All five modes deliver the notices under `licenses/cui/`. The CLI inventories
the files it delivers, including those notices, and covers them when signing.

AOT enforces linker/AOT warnings as errors in the CLI. Dynamic loading, reflection
and runtime code generation in imported libraries can make an application
unsuitable; failure never silently selects another mode. Use `single-file` when
the complete managed runtime is needed. The compiler's own
[distribution mode](distribution.md) does not determine application compatibility.

`--no-build` uses existing matching configuration/RID outputs and the SDK's
no-restore behavior. It cannot change assembly signing through `--key-file`.
`--dry-run` evaluates project/solution selection and output ownership without
publishing. Standalone sources need project preparation and cannot use dry-run.

The whole-operation `--timeout` defaults to 600 seconds, range 1..3600. Cancellation
terminates and joins owned build/linker processes. Exit codes are 0 success/preview,
1 publish/I/O/deadline/conflict failure, 2 invalid arguments or ambiguous selection,
and 130 caller cancellation. `--report json` owns stdout and includes selection,
build output, mode/RID/destination and relative file sizes/SHA-256 hashes. Normal
success prints the destination. Build logs stream to stderr during publishing.

## Output ownership

`--output` must be absent, empty, or an unchanged delivery previously produced by
this command. It must not contain the source project. Publication builds into a
private sibling directory, inventories the result, and replaces the destination
after success. Mode changes remove stale runtime files. Build failure, timeout or
cancellation preserve the previous delivery. Failed filesystem rollback retains
the previous-directory backup instead of deleting it.

`.fpas-publish.json` contains relative paths, sizes and hashes, without source
machine paths or credentials. Added, changed or missing files cause
`output_conflict`; choose another destination. Symbolic links and more than 20000
files are refused. Ownership manifests are limited to 4 MiB; CMS signatures to
8 MiB. Do not modify the destination or run programs from it during
replacement.

## Direct SDK publication

```sh
dotnet publish App/App.fpasproj -c Release -r linux-x64 -p:PublishAot=true
dotnet publish App/App.fpasproj -c Debug -r win-x64 -p:PublishSingleFile=true -p:SelfContained=true -p:IncludeNativeLibrariesForSelfExtract=true
dotnet publish App/App.fpasproj -c Release -r win-x64 -p:UseAppHost=true --self-contained false
```

Native/apphost/single-file publication requires OutputType Exe. Managed libraries
can be published without executable options. Direct `dotnet publish` retains
normal .NET output-copy behavior; the directory transaction belongs to the FPAS CLI.

## Signing

Standard `SignAssembly`, `AssemblyOriginatorKeyFile`, `PublicSign` and `DelaySign`
project properties control assembly strong names. Keys are CAPI RSA `.snk` key
pairs or strong-name public key blobs. Full signing requires a private key;
public/delay signing accepts a public key. PublicSign and DelaySign are exclusive;
key containers are unsupported. `fpas compile --key-file` exposes the same modes.
`fpas publish --key-file` applies the key to the selected build graph; project
properties can configure projects independently. Key content and mode participate
in fingerprints and analysis build proofs. Invalid keys report FP4003. Strong
names identify assemblies; they are not a security boundary or publisher trust.

For integrity of a complete delivery, including native AOT executables, use PFX:

```sh
fpas publish App/App.fpasproj --output artifacts/signed --certificate publisher.pfx --certificate-password-env PUBLISH_PASSWORD
dotnet msbuild App/App.fpasproj -t:FpasVerifyPublish -p:FpasPublishDirectory=artifacts/signed -p:FpasPublishVerificationCertificate=publisher.cer
```

Pass the password through the named environment variable. There is no plaintext
password argument. Direct SDK publication uses `FpasPublishCertificate` and
`FpasPublishCertificatePasswordEnvironment`. Private keys load ephemerally without
certificate-store installation. Selected SNK/PFX inputs are excluded from publish
copy items, including explicitly marked items.

`.fpas-signature.p7s` is CMS SignedData containing a SHA-256 inventory of all
runtime/content/symbol files. It excludes its own signature and the CLI ownership
manifest. Verification checks the cryptographic signature, the exact caller-pinned
public certificate and complete current file inventory. Added, missing or changed
files fail with FP5011. It performs no online trust lookup. CMS integrity is
distinct from OS Authenticode/notarization and trusted timestamps; normal SDK
post-publish targets remain available for those platform release tools.

## Verification

PublishingTests executes mixed FPAS/C# programs with records/domains, generics,
closures, contracts and async entry points in Debug/Release through all five
modes. Independent delivery directories have no source/compiler/feed. AOT and
single-file runs use only the executable with .NET host search disabled. The
suite also covers previews, mode changes, invalid arguments, output conflicts,
failure preservation, deadline cleanup, solution/standalone selection, no-build
reuse, independent RSA signature verification and incremental re-signing,
public/delay signing, pinned CMS certificates and tampering controls for FPAS and
C# hosts.

The [packaged GUI fixture](../../src/Fpas.Gui.Cui/README.md#packaged-publishing-fixture)
publishes and signs all five modes with the CUI binding referenced transitively.
It runs both renderers on Windows and Linux and verifies native asset placement,
extraction and static linking.

The [public GUI host](../../src/Fpas.Gui/README.md) is consumed through the
Fpas.Gui package, which brings its MVU/widget libraries and the native binding
transitively. GuiPublishingTests publishes the full FPAS gallery in all five
modes and checks controlled state, background completion/cancellation, resource
removal and repeated owner-thread teardown. Its single-file and NativeAOT cases
run with only the executable present and .NET host search disabled.

See Microsoft's [Native AOT](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/),
[single-file](https://learn.microsoft.com/en-us/dotnet/core/deploying/single-file/overview)
and [strong-name](https://learn.microsoft.com/en-us/dotnet/standard/assembly/sign-strong-name) references.
