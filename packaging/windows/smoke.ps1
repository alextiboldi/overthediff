<#
.SYNOPSIS
  Install an OverTheDiff MSI, check it, uninstall it, install it again and check what is left.

.DESCRIPTION
  For a throwaway machine such as a CI runner: it installs per machine, needs an administrator
  without a UAC prompt, and writes a marker file into the per-user settings folder.

  Checks: the install puts otd.exe, NOTICE, ATTRIBUTION.md and the licence texts in Program
  Files\OverTheDiff <major> and a Start menu entry, OverTheDiff <major>; the installed otd.exe runs
  (a launch without a window returns its exit code); the Start menu entry starts the installed
  otd.exe, which is still running 10 s later; uninstalling removes the program folder and the Start
  menu entry and keeps the settings folder; installing again finds that folder as it was.

  With -OtherMsi, an MSI of another major version, it installs that one first and also checks that
  the two install side by side (E01-S2-T5): both stay installed and start, and uninstalling the
  first leaves the other installed and starting.

.EXAMPLE
  pwsh packaging/windows/smoke.ps1 -Msi dist\release\overthediff-0.1.0-windows-x64.msi -Version 0.1.0

.EXAMPLE
  pwsh packaging/windows/smoke.ps1 -Msi overthediff-1.0.0-windows-x64.msi -Version 1.0.0 -OtherMsi overthediff-0.1.0-windows-x64.msi -OtherVersion 0.1.0
#>
param(
  [Parameter(Mandatory)] [string] $Msi,
  [Parameter(Mandatory)] [string] $Version,
  [string] $OtherMsi,
  [string] $OtherVersion
)
$ErrorActionPreference = 'Stop'
$Log = Join-Path ([IO.Path]::GetTempPath()) 'overthediff-msi'
New-Item -ItemType Directory -Force -Path $Log | Out-Null

function Check([bool] $Ok, [string] $What) {
  if (-not $Ok) { throw "smoke test failed: $What" }
  Write-Output "ok $What"
}

# Where the MSI of a version installs, and otd-settings' folder for it (overthediff-<major> under
# %APPDATA%).
function Get-Install([string] $Msi, [string] $Version) {
  $major = ($Version -split '\.')[0]
  $dir = Join-Path $env:ProgramFiles "OverTheDiff $major"
  [pscustomobject]@{
    Msi      = (Resolve-Path $Msi).Path
    Major    = $major
    Dir      = $dir
    Exe      = Join-Path $dir 'otd.exe'
    Shortcut = Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\OverTheDiff $major.lnk"
    Settings = Join-Path $env:APPDATA "overthediff-$major"
  }
}

function Invoke-Msiexec([string] $Mode, $Install, [string] $LogName) {
  $logFile = Join-Path $Log $LogName
  $p = Start-Process msiexec.exe -ArgumentList $Mode, "`"$($Install.Msi)`"", '/qn', '/l*v', "`"$logFile`"" -Wait -PassThru
  if ($p.ExitCode -ne 0 -and $p.ExitCode -ne 3010) {
    Get-Content $logFile -Tail 60 -ErrorAction SilentlyContinue
    throw "msiexec $Mode $($Install.Msi) exited with $($p.ExitCode)"
  }
}

function Test-Installed($Install) {
  $files = 'otd.exe', 'NOTICE', 'ATTRIBUTION.md', 'licences\UFL.txt', 'licences\Hack-Regular.txt',
    'licences\emoji-icon-font-mit-license.txt', 'licences\option-ext-LICENSE.txt'
  foreach ($f in $files) { Check (Test-Path (Join-Path $Install.Dir $f)) "installed $($Install.Dir)\$f" }
  Check (Test-Path $Install.Shortcut) "Start menu entry $($Install.Shortcut) exists"
}

function Test-Start($Install) {
  # An unclaimed argument exits with 100 (otd-launch's EXIT_UNCLAIMED) before any window opens:
  # the installed exe loads and runs on a machine with only what the MSI put there.
  $p = Start-Process $Install.Exe -ArgumentList '/no-such-switch' -Wait -PassThru
  Check ($p.ExitCode -eq 100) "installed $($Install.Exe) runs (exit $($p.ExitCode), expected 100)"

  Write-Output "==> open the window from $($Install.Shortcut)"
  Start-Process $Install.Shortcut
  Start-Sleep -Seconds 10
  $running = @(Get-Process otd -ErrorAction SilentlyContinue)
  $mine = @($running | Where-Object Path -EQ $Install.Exe)
  Check ($mine.Count -ge 1) "the Start menu entry started $($Install.Exe), still running 10 s later (running: $($running.Path -join ', '))"
  $mine | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 2
}

$Main = Get-Install $Msi $Version
$Other = $null
if ($OtherMsi) {
  if (-not $OtherVersion) { throw '-OtherMsi needs -OtherVersion' }
  $Other = Get-Install $OtherMsi $OtherVersion
  if ($Other.Major -eq $Main.Major) { throw "$Msi and $OtherMsi are both major version $($Main.Major)" }
  Write-Output "==> install the other major version, $($Other.Msi)"
  Invoke-Msiexec '/i' $Other 'install-other.log'
}

Write-Output "==> install $($Main.Msi)"
Invoke-Msiexec '/i' $Main 'install.log'
Test-Installed $Main
Test-Start $Main
if ($Other) {
  Write-Output '==> the other major version is still installed and starts'
  Test-Installed $Other
  Test-Start $Other
}

# The user's settings, which uninstalling must keep.
New-Item -ItemType Directory -Force -Path $Main.Settings | Out-Null
$Marker = Join-Path $Main.Settings 'smoke-test-marker.txt'
Set-Content -Path $Marker -Value 'kept across uninstall'

Write-Output '==> uninstall'
Invoke-Msiexec '/x' $Main 'uninstall.log'
Check (-not (Test-Path $Main.Dir)) 'program folder removed'
Check (-not (Test-Path $Main.Shortcut)) 'Start menu entry removed'
Check (Test-Path $Marker) 'settings folder kept'
if ($Other) {
  Write-Output '==> the other major version still starts'
  Test-Installed $Other
  Test-Start $Other
}

Write-Output '==> install it again'
Invoke-Msiexec '/i' $Main 'reinstall.log'
Test-Installed $Main
Check (Test-Path $Marker) 'the reinstalled version finds its settings folder as it was'
Invoke-Msiexec '/x' $Main 'uninstall-again.log'
Remove-Item -Force $Marker
Write-Output 'smoke test passed'
