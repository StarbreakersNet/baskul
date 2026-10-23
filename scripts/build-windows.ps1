# Construit dist\Baskul (baskul.exe + baskul-settings.exe) et
# dist\Baskul-Windows.zip. Nécessite Rust et Node.js.
#   powershell -ExecutionPolicy Bypass -File scripts\build-windows.ps1
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

function Run($exe) {
  & $exe @args
  if ($LASTEXITCODE -ne 0) { throw "échec : $exe $args" }
}

# Interface de réglages, intégrée ensuite à l'exécutable Tauri.
Push-Location app
Run corepack yarn install --immutable
Run corepack yarn build
Pop-Location

Run cargo build --release -p baskul
Run cargo build --release -p baskul-settings --features custom-protocol

$out = "dist\Baskul"
if (Test-Path dist) { Remove-Item -Recurse -Force dist }
New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item target\release\baskul.exe, target\release\baskul-settings.exe $out
Compress-Archive -Path $out -DestinationPath dist\Baskul-Windows.zip -Force
Write-Host ""
Write-Host "OK : $out (baskul.exe + baskul-settings.exe) et dist\Baskul-Windows.zip"
