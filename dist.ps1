# Build release binary and bundled standard-library sources in bin/.
cargo build --release -p fpas-cli
cargo run --release -p fpas-build --example precompile_stdlib -- target\release\lib
New-Item -ItemType Directory -Path bin -Force | Out-Null
New-Item -ItemType Directory -Path bin\lib -Force | Out-Null
Copy-Item target\release\fpas.exe bin\fpas.exe -Force
Copy-Item target\release\lib\* bin\lib -Recurse -Force
Write-Host "Built: bin\fpas.exe and bin\lib"
