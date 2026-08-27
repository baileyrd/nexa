# Deterministic clean-checkout validation for a representative CPU-only Windows PC.
# Run from any directory: powershell -File spikes/g1-shared-ui/scripts/validate-windows.ps1
[CmdletBinding()]
param([string]$OutputPath = "")
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if (-not $OutputPath) { $OutputPath = Join-Path $root "evidence/windows-validation.json" }
$steps = [System.Collections.Generic.List[object]]::new()
$started = [DateTime]::UtcNow

function Invoke-ValidationStep {
  param([string]$Name, [string]$WorkingDirectory, [string]$Executable, [string[]]$Arguments)
  $stepStarted = [DateTime]::UtcNow
  Push-Location $WorkingDirectory
  try {
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Executable exited with code $LASTEXITCODE" }
    $steps.Add([ordered]@{ name=$Name; status="pass"; duration_seconds=[Math]::Round(([DateTime]::UtcNow-$stepStarted).TotalSeconds,3) })
  } catch {
    $steps.Add([ordered]@{ name=$Name; status="fail"; duration_seconds=[Math]::Round(([DateTime]::UtcNow-$stepStarted).TotalSeconds,3); error=$_.Exception.Message })
    throw
  } finally { Pop-Location }
}

$status = "pass"
$errorMessage = $null
$packages = @()
try {
  Invoke-ValidationStep "frontend_npm_ci" "$root/frontend" "npm.cmd" @("ci")
  Invoke-ValidationStep "frontend_check" "$root/frontend" "npm.cmd" @("run", "check")
  Invoke-ValidationStep "frontend_component_tests" "$root/frontend" "npm.cmd" @("test")
  Invoke-ValidationStep "frontend_production_build" "$root/frontend" "npm.cmd" @("run", "build")
  Invoke-ValidationStep "runtime_format" "$root/runtime" "cargo.exe" @("fmt", "--check")
  Invoke-ValidationStep "runtime_check" "$root/runtime" "cargo.exe" @("check", "--locked")
  Invoke-ValidationStep "runtime_tests" "$root/runtime" "cargo.exe" @("test", "--locked")
  Invoke-ValidationStep "desktop_npm_ci" "$root/desktop" "npm.cmd" @("ci")
  Invoke-ValidationStep "tauri_windows_build_package" "$root/desktop" "npm.cmd" @("run", "tauri", "--", "build")
  $bundleRoot = Join-Path $root "desktop/src-tauri/target/release/bundle"
  $packages = @(Get-ChildItem $bundleRoot -File -Recurse | Sort-Object FullName | ForEach-Object {
    [ordered]@{ path=$_.FullName.Substring($root.Length + 1).Replace("\", "/"); bytes=$_.Length }
  })
  if ($packages.Count -eq 0) { throw "Tauri build produced no package files under $bundleRoot" }
} catch {
  $status = "fail"
  $errorMessage = $_.Exception.Message
}
$result = [ordered]@{
  schema_version = 1
  gate = "G1"
  platform = [System.Environment]::OSVersion.VersionString
  architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
  cpu_only_required = $true
  started_utc = $started.ToString("o")
  finished_utc = [DateTime]::UtcNow.ToString("o")
  status = $status
  steps = $steps
  packages = $packages
  limitations = @("This script does not prove interactive browser/WebView parity, accessibility, startup, CPU, memory, reconnect timing, or cancellation timing.")
}
if ($errorMessage) { $result["error"] = $errorMessage }
$json = $result | ConvertTo-Json -Depth 8
New-Item -ItemType Directory -Force (Split-Path $OutputPath) | Out-Null
[System.IO.File]::WriteAllText($OutputPath, $json + [Environment]::NewLine, [System.Text.UTF8Encoding]::new($false))
Write-Output $json
if ($status -ne "pass") { exit 1 }
