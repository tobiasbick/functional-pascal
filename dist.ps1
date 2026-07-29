# Build release binaries and bundled standard-library sources in bin/.
cargo build --release -p fpas-cli
cargo run --release -p fpas-build --example precompile_stdlib -- target\release\lib bin\lib
New-Item -ItemType Directory -Path bin -Force | Out-Null
Copy-Item target\release\fpas.exe bin\fpas.exe -Force
Copy-Item target\release\fpas-runner.exe bin\fpas-runner.exe -Force
Write-Host "Built: bin\fpas.exe, bin\fpas-runner.exe, and bin\lib"
