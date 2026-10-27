# Verify distribution failure handling with an isolated copy of the built compiler.
param(
    [string]$ScriptPath = (Join-Path $PSScriptRoot '../../dist.ps1'),
    [string]$FpasPath = (Join-Path $PSScriptRoot '../../target/debug/fpas.exe')
)
$ErrorActionPreference = 'Stop'
$scriptSource = Get-Content -Raw -LiteralPath $ScriptPath
$compiler = (Resolve-Path -LiteralPath $FpasPath).Path
$fixture = Join-Path ([System.IO.Path]::GetTempPath()) ('fpas-dist-test-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $fixture | Out-Null
$fixture = (Resolve-Path -LiteralPath $fixture).Path
$initialLocation = (Get-Location).Path

function cargo {
    $global:fpasDistTestCargoArguments += ,(($args | ForEach-Object { "$_" }) -join ' ')
    $global:LASTEXITCODE = if ($global:fpasDistTestFailure -eq 'build') { 42 } else { 0 }
}

try {
    Set-Content -LiteralPath (Join-Path $fixture 'dist.ps1') -Value $scriptSource
    New-Item -ItemType Directory -Path (Join-Path $fixture 'target/release/lib/Std') -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $fixture 'bin') -Force | Out-Null
    Copy-Item -LiteralPath $compiler -Destination (Join-Path $fixture 'target/release/fpas.exe')
    Set-Content -LiteralPath (Join-Path $fixture 'target/release/lib/stdlib.fpasprj') -Value @'
[project]
name = "distribution-fixture"
kind = "library"
[exports]
units = ["Std.DistributionFixture"]
[sources]
include = ["Std/**/*.fpas"]
'@
    foreach ($failure in @('build', 'stage', 'success')) {
        $global:fpasDistTestFailure = $failure
        $global:fpasDistTestCargoArguments = @()
        $source = if ($failure -eq 'stage') { 'invalid Pascal' } else {
            'unit Std.DistributionFixture; const Answer: integer := 42; end unit;'
        }
        Set-Content -LiteralPath (Join-Path $fixture 'target/release/lib/Std/DistributionFixture.fpas') -Value $source
        foreach ($binary in @('fpas.exe', 'fpas-runner.exe', 'fpas-lsp.exe')) {
            if ($binary -ne 'fpas.exe') {
                Set-Content -LiteralPath (Join-Path $fixture "target/release/$binary") -Value 'new'
            }
            Set-Content -LiteralPath (Join-Path $fixture "bin/$binary") -Value 'old'
        }
        $failed = $false
        $output = @()
        try {
            & (Join-Path $fixture 'dist.ps1') 6>&1 | ForEach-Object { $output += "$_" }
        }
        catch {
            $failed = $true
        }
        if ($failed -ne ($failure -ne 'success')) { throw "Wrong failure outcome for $failure" }
        if ($global:fpasDistTestCargoArguments.Count -ne 1) { throw 'Expected one Cargo invocation' }
        if ($global:fpasDistTestCargoArguments[0] -ne 'build --locked --release -p fpas-cli -p fpas-lsp') {
            throw 'Incorrect release build arguments'
        }
        $reportedSuccess = [bool]($output -match '^Built:')
        if ($reportedSuccess -ne ($failure -eq 'success')) { throw 'Incorrect success report' }
        foreach ($binary in @('fpas.exe', 'fpas-runner.exe', 'fpas-lsp.exe')) {
            $published = Join-Path $fixture "bin/$binary"
            if ($failure -eq 'success') {
                $expected = (Get-FileHash -LiteralPath (Join-Path $fixture "target/release/$binary")).Hash
                if ((Get-FileHash -LiteralPath $published).Hash -ne $expected) { throw "Incorrect binary: $binary" }
            }
            elseif ((Get-Content -Raw -LiteralPath $published).Trim() -ne 'old') {
                throw "Published binary after failure: $binary"
            }
        }
        $sidecar = Join-Path $fixture 'bin/lib/Std/DistributionFixture.fpascu'
        if ((Test-Path -LiteralPath $sidecar) -ne ($failure -eq 'success')) { throw 'Incorrect staged library' }
        if ((Get-Location).Path -ne $initialLocation) { throw 'Working directory was not restored' }
        Write-Output "PASS: $failure"
    }
}
finally {
    $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\', '/')
    if (-not $fixture.StartsWith($tempRoot + [System.IO.Path]::DirectorySeparatorChar)) {
        throw 'Fixture must be inside the temporary directory'
    }
    Remove-Item -LiteralPath $fixture -Recurse -Force
}
