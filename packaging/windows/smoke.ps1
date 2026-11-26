<#
.SYNOPSIS
  Install an OverTheDiff MSI, check it, uninstall it and check what is left.

.DESCRIPTION
  For a throwaway machine such as a CI runner: it installs per machine, needs an administrator
  without a UAC prompt, and writes a marker file into the per-user settings folder.

  Checks: the install puts otd.exe, NOTICE and ATTRIBUTION.md in Program Files\OverTheDiff and a
  Start menu entry; the installed otd.exe runs (a launch without a window
  returns its exit code); uninstalling removes the program folder and the Start menu entry and
  keeps the settings folder. Opening the window from the Start menu entry is reported only: a
  hosted runner may have no graphics adapter, so that check is done by hand.

.EXAMPLE
  pwsh packaging/windows/smoke.ps1 -Msi dist\release\overthediff-0.1.0-windows-x64.msi -Version 0.1.0
#>
param(
  [Parameter(Mandatory)] [string] $Msi,
  [Parameter(Mandatory)] [string] $Version
)
$ErrorActionPreference = 'Stop'
$Msi = (Resolve-Path $Msi).Path
$InstallDir = Join-Path $env:ProgramFiles 'OverTheDiff'
$Shortcut = Join-Path $env:ProgramData 'Microsoft\Windows\Start Menu\Programs\OverTheDiff.lnk'
$Exe = Join-Path $InstallDir 'otd.exe'
# otd-settings' folder: overthediff-<major> under %APPDATA%.
$SettingsDir = Join-Path $env:APPDATA ('overthediff-' + ($Version -split '\.')[0])
$Log = Join-Path ([IO.Path]::GetTempPath()) 'overthediff-msi'
New-Item -ItemType Directory -Force -Path $Log | Out-Null

function Check([bool] $Ok, [string] $What) {
  if (-not $Ok) { throw "smoke test failed: $What" }
  Write-Output "ok $What"
}

function Invoke-Msiexec([string] $Mode, [string] $LogName) {
  $p = Start-Process msiexec.exe -ArgumentList $Mode, "`"$Msi`"", '/qn', '/l*v', "`"$(Join-Path $Log $LogName)`"" -Wait -PassThru
  if ($p.ExitCode -ne 0 -and $p.ExitCode -ne 3010) {
    Get-Content (Join-Path $Log $LogName) -Tail 60 -ErrorAction SilentlyContinue
    throw "msiexec $Mode exited with $($p.ExitCode)"
  }
}

Write-Output "==> install $Msi"
Invoke-Msiexec '/i' 'install.log'
foreach ($f in 'otd.exe', 'NOTICE', 'ATTRIBUTION.md') { Check (Test-Path (Join-Path $InstallDir $f)) "installed $f" }
Check (Test-Path $Shortcut) 'Start menu entry exists'

# An unclaimed argument exits with 100 (otd-launch's EXIT_UNCLAIMED) before any window opens: the
# installed exe loads and runs on a machine with only what the MSI put there.
$p = Start-Process $Exe -ArgumentList '/no-such-switch' -Wait -PassThru
Check ($p.ExitCode -eq 100) "installed otd.exe runs (exit $($p.ExitCode), expected 100)"

Write-Output '==> open the window from the Start menu entry (reported, not checked)'
Start-Process $Shortcut
Start-Sleep -Seconds 10
$running = Get-Process otd -ErrorAction SilentlyContinue
if ($running) { Write-Output 'report: otd is running after a Start menu launch' } else { Write-Output 'report: otd is not running 10 s after a Start menu launch (no graphics adapter?)' }
$running | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# The user's settings, which uninstalling must keep.
New-Item -ItemType Directory -Force -Path $SettingsDir | Out-Null
$Marker = Join-Path $SettingsDir 'smoke-test-marker.txt'
Set-Content -Path $Marker -Value 'kept across uninstall'

Write-Output '==> uninstall'
Invoke-Msiexec '/x' 'uninstall.log'
Check (-not (Test-Path $InstallDir)) 'program folder removed'
Check (-not (Test-Path $Shortcut)) 'Start menu entry removed'
Check (Test-Path $Marker) 'settings folder kept'
Remove-Item -Force $Marker
Write-Output 'smoke test passed'
