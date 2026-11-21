# [Documentation](../../docs/build/distribution.md#windows-portable-launcher)
param(
    [Parameter(Mandatory = $true)][string]$PayloadDirectory,
    [Parameter(Mandatory = $true)][string]$EntryPoint,
    [Parameter(Mandatory = $true)][string]$Output,
    [Parameter(Mandatory = $true)][string]$ArtifactsPath,
    [string]$Icon
)

$ErrorActionPreference = 'Stop'
$outputPath = [IO.Path]::GetFullPath($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Output))
$name = [IO.Path]::GetFileNameWithoutExtension($outputPath)
$payload = (Resolve-Path -LiteralPath $PayloadDirectory).Path.TrimEnd([IO.Path]::DirectorySeparatorChar)
if ([IO.Path]::GetFileName($EntryPoint) -ne $EntryPoint -or
    -not (Test-Path -LiteralPath (Join-Path $payload $EntryPoint) -PathType Leaf)) {
    throw 'The portable payload must contain the named executable.'
}

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$artifacts = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($ArtifactsPath)
[IO.Directory]::CreateDirectory($artifacts) | Out-Null
$work = Join-Path $artifacts ([Guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($work) | Out-Null
$archivePath = Join-Path $work 'payload.zip'
$manifestPath = Join-Path $work 'payload.txt'

$archive = [IO.Compression.ZipFile]::Open($archivePath, [IO.Compression.ZipArchiveMode]::Create)
$sha256 = [Security.Cryptography.SHA256]::Create()
try {
    $manifest = @($EntryPoint)
    foreach ($file in (Get-ChildItem -LiteralPath $payload -Recurse -File | Sort-Object FullName)) {
        if ($file.Extension -eq '.pdb') { continue }
        $relative = $file.FullName.Substring($payload.Length + 1).Replace('\', '/')
        $inputFile = $file.OpenRead()
        try { $hash = [BitConverter]::ToString($sha256.ComputeHash($inputFile)).Replace('-', '') }
        finally { $inputFile.Dispose() }
        $manifest += "$hash $relative"
        [IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
            $archive, $file.FullName, $relative, [IO.Compression.CompressionLevel]::Optimal
        ) | Out-Null
    }
    [IO.File]::WriteAllLines($manifestPath, [string[]]$manifest)
}
finally {
    $sha256.Dispose()
    $archive.Dispose()
}

$dist = Join-Path $work 'publish'
$properties = @("-p:PortableName=$name", "-p:PortableArchive=$archivePath", "-p:PortableManifest=$manifestPath")
if ($Icon) {
    $properties += '-p:PortableGui=true'
    $properties += "-p:PortableIcon=$((Resolve-Path -LiteralPath $Icon).Path)"
}
dotnet publish (Join-Path $PSScriptRoot 'Fpas.WindowsPortable.csproj') -c Release -r win-x64 `
    --artifacts-path (Join-Path $work 'build') --output $dist @properties --nologo -warnaserror
if ($LASTEXITCODE -ne 0) { throw "Building the portable launcher failed (exit code $LASTEXITCODE)." }

[IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($outputPath)) | Out-Null
Copy-Item -LiteralPath (Join-Path $dist "$name.exe") -Destination $outputPath -Force
