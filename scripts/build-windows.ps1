# Construit dist\Baskul (baskul.exe : Baskul et sa fenêtre de réglages, avec
# Tauri) et dist\Baskul-Windows.zip. Nécessite Rust et Node.js.
#   powershell -ExecutionPolicy Bypass -File scripts\build-windows.ps1
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

function Run($exe) {
  & $exe @args
  if ($LASTEXITCODE -ne 0) { throw "échec : $exe $args" }
}

# Front construit par beforeBuildCommand, puis intégré à l'exécutable.
Push-Location app
Run corepack yarn install --immutable
Run corepack yarn tauri build --no-bundle
Pop-Location

$out = "dist\Baskul"
if (Test-Path dist) { Remove-Item -Recurse -Force dist }
New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item target\release\baskul.exe $out
Compress-Archive -Path $out -DestinationPath dist\Baskul-Windows.zip -Force
Write-Host ""
Write-Host "OK : $out (baskul.exe) et dist\Baskul-Windows.zip"
