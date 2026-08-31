param(
  [Parameter(Mandatory=$true)][string]$Config,
  [Parameter(Mandatory=$true)][string]$ModelManifest
)
$ErrorActionPreference = "Stop"
function Invoke-Native {
  param([Parameter(Mandatory=$true)][string]$Step,
        [Parameter(Mandatory=$true)][scriptblock]$Command)
  $output = & $Command
  if ($LASTEXITCODE -ne 0) { throw "Native command failed during '$Step' (exit $LASTEXITCODE)" }
  return $output
}
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root
if (-not $IsWindows) { throw "G2 owner evidence must run on Windows" }
$Preflight = Invoke-Native "repository preflight" { git status --short --untracked-files=all }
if ($Preflight) { throw "Tracked or non-ignored changes existed before validation:`n$Preflight" }
Invoke-Native "create virtual environment" { py -3.11 -m venv .venv }
Invoke-Native "install pinned pip" { & .\.venv\Scripts\python -m pip install --upgrade pip==26.0.1 }
Invoke-Native "install dependencies" { & .\.venv\Scripts\python -m pip install -r requirements.txt }
$env:PYTHONPATH = $Root
Invoke-Native "offline unit tests" { & .\.venv\Scripts\python -m unittest discover -s tests -v }
New-Item -ItemType Directory -Force evidence\windows | Out-Null
Invoke-Native "provenance and footprint" { & .\.venv\Scripts\python -m g2_spike.evidence --manifest $ModelManifest --root . --venv .venv --output evidence\windows\footprint.json }
Invoke-Native "device evidence" { & .\.venv\Scripts\python -m g2_spike.devices --record evidence\windows\microphone.wav --play evidence\windows\prompt.wav --report evidence\windows\devices.json }
Invoke-Native "speech evidence" { & .\.venv\Scripts\python -m g2_spike.run --config $Config --fixtures fixtures.json --output evidence\windows\automated.json }
Invoke-Native "capture cancellation" { & .\.venv\Scripts\python -m g2_spike.trial --config $Config --stage capture --output evidence\windows\cancel-capture.wav --cancel-after 0.25 --report evidence\windows\cancel-capture.json }
Invoke-Native "recognition cancellation" { & .\.venv\Scripts\python -m g2_spike.trial --config $Config --stage recognition --input fixtures\tcp-01.wav --cancel-after 0.01 --report evidence\windows\cancel-recognition.json }
Invoke-Native "synthesis cancellation" { & .\.venv\Scripts\python -m g2_spike.trial --config $Config --stage synthesis --output evidence\windows\cancel-synthesis.wav --cancel-after 0.01 --report evidence\windows\cancel-synthesis.json }
Invoke-Native "playback cancellation" { & .\.venv\Scripts\python -m g2_spike.trial --config $Config --stage playback --input evidence\windows\prompt.wav --cancel-after 0.25 --report evidence\windows\cancel-playback.json }
Get-ComputerInfo | Out-File evidence\windows\computer-info.txt
$PostRun = Invoke-Native "repository postflight" { git status --short --untracked-files=all }
@{ schema = 1; preflight_clean = $true; post_run_clean = (-not $PostRun); generated_evidence_ignored = (-not $PostRun); unexpected_changes = @($PostRun) } |
  ConvertTo-Json | Set-Content evidence\windows\repository-state.json
if ($PostRun) { throw "Validation changed tracked or non-ignored paths:`n$PostRun" }
Write-Host "Automation finished. Complete the owner-observation table in evidence/G2-EVIDENCE.md; this script does not pass G2."
