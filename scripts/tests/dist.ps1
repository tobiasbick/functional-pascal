# Verify distribution failure handling without invoking Cargo or replacing bin/.
param([string]$ScriptPath = (Join-Path $PSScriptRoot '../../dist.ps1'))
$ErrorActionPreference = 'Stop'
$scriptSource = Get-Content -Raw -LiteralPath $ScriptPath
$fixture = Join-Path ([System.IO.Path]::GetTempPath()) ('fpas-dist-test-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $fixture | Out-Null
$fixture = (Resolve-Path -LiteralPath $fixture).Path
$initialLocation = (Get-Location).Path

function cargo {
    $global:fpasDistTestCargoCalls++
    $global:LASTEXITCODE = 0
    if ($global:fpasDistTestCargoCalls -eq $global:fpasDistTestFailCall) {
        $global:LASTEXITCODE = 42
    }
}

try {
    Set-Content -LiteralPath (Join-Path $fixture 'dist.ps1') -Value $scriptSource
    New-Item -ItemType Directory -Path (Join-Path $fixture 'target/release') -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $fixture 'bin') -Force | Out-Null
    foreach ($failure in @(1, 2, 0)) {
        $global:fpasDistTestFailCall = $failure
        $global:fpasDistTestCargoCalls = 0
        foreach ($binary in @('fpas.exe', 'fpas-runner.exe')) {
            Set-Content -LiteralPath (Join-Path $fixture "target/release/$binary") -Value 'new'
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
        if ($failed -ne ($failure -ne 0)) { throw "Wrong failure outcome for Cargo call $failure" }
        $expectedCalls = if ($failure -eq 1) { 1 } else { 2 }
        if ($global:fpasDistTestCargoCalls -ne $expectedCalls) { throw 'Cargo continued after failure' }
        $reportedSuccess = [bool]($output -match '^Built:')
        if ($reportedSuccess -ne ($failure -eq 0)) { throw 'Incorrect success report' }
        $expectedBinary = if ($failure -eq 0) { 'new' } else { 'old' }
        foreach ($binary in @('fpas.exe', 'fpas-runner.exe')) {
            $actual = (Get-Content -Raw -LiteralPath (Join-Path $fixture "bin/$binary")).Trim()
            if ($actual -ne $expectedBinary) { throw "Incorrect published binary: $binary" }
        }
        if ((Get-Location).Path -ne $initialLocation) { throw 'Working directory was not restored' }
        Write-Output "PASS: Cargo failure call $failure"
    }
}
finally {
    $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\', '/')
    if (-not $fixture.StartsWith($tempRoot + [System.IO.Path]::DirectorySeparatorChar)) {
        throw 'Fixture must be inside the temporary directory'
    }
    Remove-Item -LiteralPath $fixture -Recurse -Force
}
