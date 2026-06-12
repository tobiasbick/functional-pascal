# Formats all `.fpas` sources under examples/, tests/, and apps/ (skips target/).
# Check only: scripts/format-fpas-sources.ps1 -Check
# List dirty paths: scripts/format-fpas-sources.ps1 -Check -List
param(
    [switch]$Check,
    [switch]$List
)

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$args = @()
if ($Check) { $args += "--check" }
if ($List) { $args += "--list" }

cargo run -q -p fpas-cli -- fmt @args examples tests apps
