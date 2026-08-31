param(
  [Parameter(Mandatory=$true)][string]$Config,
  [Parameter(Mandatory=$true)][string]$ModelManifest
)
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root
if (-not $IsWindows) { throw "G2 owner evidence must run on Windows" }
$manifest = Get-Content $ModelManifest -Raw | ConvertFrom-Json
foreach ($model in $manifest.models) {
  if ($model.source -eq "REQUIRED" -or $model.license -eq "REQUIRED" -or $model.sha256 -eq "REQUIRED") {
    throw "Model manifest has unresolved provenance fields"
  }
}
py -3.11 -m venv .venv
& .\.venv\Scripts\python -m pip install --upgrade pip==26.0.1
& .\.venv\Scripts\python -m pip install -r requirements.txt
$env:PYTHONPATH = $Root
& .\.venv\Scripts\python -m unittest discover -s tests -v
New-Item -ItemType Directory -Force evidence\windows | Out-Null
& .\.venv\Scripts\python -m g2_spike.devices --record evidence\windows\microphone.wav --play evidence\windows\prompt.wav --report evidence\windows\devices.json
& .\.venv\Scripts\python -m g2_spike.run --config $Config --fixtures fixtures.json --output evidence\windows\automated.json
Get-FileHash -Algorithm SHA256 (Get-ChildItem models -Recurse -File) | ConvertTo-Json | Set-Content evidence\windows\model-hashes.json
Get-ComputerInfo | Out-File evidence\windows\computer-info.txt
git status --short | Out-File evidence\windows\git-status.txt
Write-Host "Automation finished. Complete the owner-observation table in evidence/G2-EVIDENCE.md; this script does not pass G2."

