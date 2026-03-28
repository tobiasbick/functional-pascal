# Build release binary and copy to bin/.
cargo build --release -p fpas-cli
New-Item -ItemType Directory -Path bin -Force | Out-Null
Copy-Item target\release\fpas.exe bin\fpas.exe -Force
Write-Host "Built: bin\fpas.exe"
