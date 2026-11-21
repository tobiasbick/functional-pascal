# [Documentation](../../doc/desktop.md#windows-portable-build)
param(
    [Parameter(Mandatory = $true)][string]$Payload,
    [Parameter(Mandatory = $true)][string]$Cache
)

$ErrorActionPreference = 'Stop'
function Get-ArchiveDigest([string]$Path) {
    $sha256 = [Security.Cryptography.SHA256]::Create()
    try {
        $inputFile = [IO.File]::OpenRead($Path)
        try { [BitConverter]::ToString($sha256.ComputeHash($inputFile)).Replace('-', '') }
        finally { $inputFile.Dispose() }
    }
    finally { $sha256.Dispose() }
}

$stem = 'ffmpeg-n8.1.3-39-g660b5aa7db-win64-lgpl-8.1'
$tag = 'autobuild-2026-11-20-12-40'
$inputs = @(
    @{ Name = "$stem.zip"
       Url = "https://github.com/FFmpeg-Builds/FFmpeg-Builds/releases/download/$tag/$stem.zip"
       Hash = 'A16D63673D8B28EC0B4FB7311677FD8F6FA9F221F40BF00B48C01CE8E9122BE8' },
    @{ Name = 'ffmpeg-660b5aa7db.tar.gz'
       Url = 'https://codeload.github.com/FFmpeg/FFmpeg/tar.gz/660b5aa7db'
       Hash = 'E6733AD9759FB53B9A5CC18FC99D36FCA0C04B54612D799D202BF3F3594E7E6B' },
    @{ Name = 'ffmpeg-builds-2026-11-20-12-40.tar.gz'
       Url = "https://codeload.github.com/FFmpeg-Builds/FFmpeg-Builds/tar.gz/refs/tags/$tag"
       Hash = '28DC02B5472D0E751B83FE7C34E27D1E4411E278C42AEB2B13E0B7CB416E094C' }
)
New-Item -ItemType Directory -Force -Path $Cache | Out-Null
foreach ($item in $inputs) {
    $path = Join-Path $Cache $item.Name
    if (-not (Test-Path -LiteralPath $path) -or (Get-ArchiveDigest $path) -ne $item.Hash) {
        $download = "$path.$([Guid]::NewGuid().ToString('N')).tmp"
        try {
            curl.exe --fail --location --silent --show-error --retry 2 --output $download $item.Url
            if ($LASTEXITCODE -ne 0) { throw "Downloading $($item.Name) failed." }
            if ((Get-ArchiveDigest $download) -ne $item.Hash) { throw "Incorrect digest for $($item.Name)." }
            Move-Item -LiteralPath $download -Destination $path -Force
        }
        finally {
            if (Test-Path -LiteralPath $download) { Remove-Item -LiteralPath $download }
        }
    }
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
$tools = Join-Path $Payload 'tools/ffmpeg'
$licenses = Join-Path $Payload 'licenses/ffmpeg'
New-Item -ItemType Directory -Force -Path $tools, $licenses | Out-Null
$archive = [IO.Compression.ZipFile]::OpenRead((Join-Path $Cache "$stem.zip"))
try {
    [IO.Compression.ZipFileExtensions]::ExtractToFile($archive.GetEntry("$stem/bin/ffmpeg.exe"), (Join-Path $tools 'ffmpeg.exe'), $true)
    [IO.Compression.ZipFileExtensions]::ExtractToFile($archive.GetEntry("$stem/LICENSE.txt"), (Join-Path $licenses 'LICENSE.txt'), $true)
}
finally {
    $archive.Dispose()
}
foreach ($item in ($inputs | Select-Object -Skip 1)) {
    Copy-Item -LiteralPath (Join-Path $Cache $item.Name) -Destination $licenses -Force
}

# The miniaudio DLL in the current SoundFlow package uses the shared VC runtime.
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $vs) { throw 'The Visual Studio C++ build tools are required.' }
$runtime = Get-ChildItem -LiteralPath (Join-Path $vs 'VC/Redist/MSVC') -Directory |
    Where-Object { $_.Name -as [version] } |
    Sort-Object { [version]$_.Name } -Descending |
    ForEach-Object { Get-Item -Path (Join-Path $_.FullName 'x64/Microsoft.VC*.CRT/vcruntime140.dll') -ErrorAction SilentlyContinue } |
    Select-Object -First 1
if (-not $runtime) { throw 'The app-local x64 Visual C++ runtime was not found under the Visual Studio Redist directory.' }
Copy-Item -LiteralPath $runtime.FullName -Destination $Payload -Force
