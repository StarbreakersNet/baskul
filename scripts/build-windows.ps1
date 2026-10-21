# Construit dist\Bascule.exe
#   powershell -ExecutionPolicy Bypass -File scripts\build-windows.ps1
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "échec de la compilation" }
New-Item -ItemType Directory -Force dist | Out-Null
Copy-Item target\release\bascule.exe dist\Bascule.exe -Force
Write-Host ""
Write-Host "OK : dist\Bascule.exe"
