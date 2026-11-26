<#
.SYNOPSIS
  Build and package OverTheDiff for Windows as an MSI.

.DESCRIPTION
  Writes overthediff-<version>-windows-<arch>.msi to $env:DIST (default: dist\release). The shared
  release workflow (storytold/craft-common-actions release.yml) runs it once per architecture.
  The MSI is unsigned: signing waits for the maintainer's go-ahead (AGENTS.md rule 4).

  otd.exe links the C runtime statically, so a clean Windows needs no Visual C++ redistributable.
  Per-monitor DPI awareness (v2) comes from winit, which sets it before the first window opens.

  On GitHub Actions it then runs smoke.ps1 on the new MSI: install, check, uninstall.

  Needs: Rust with the MSVC toolchain and the target, and WiX v5
  (dotnet tool install --global wix --version 5.0.2).

.EXAMPLE
  pwsh packaging/windows/package.ps1 -Arch x64
#>
param(
  [ValidateSet('x64', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# The release workflow exports OVERTHEDIFF_VERSION; otherwise [workspace.package] version.
$Version = $env:OVERTHEDIFF_VERSION
if (-not $Version) {
  $inPkg = $false
  foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
    if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
  }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }
# An MSI version is numeric (major.minor.build), so a prerelease tag is dropped there.
$MsiVersion = ($Version -split '[-+]')[0]

$Target = @{ x64 = 'x86_64-pc-windows-msvc'; arm64 = 'aarch64-pc-windows-msvc' }[$Arch]
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null
Write-Output "OverTheDiff $Version for Windows $Arch ($Target)"

if (-not $SkipBuild) {
  # Static CRT, scoped to the target so build scripts and proc-macros are unaffected.
  $flagVar = 'CARGO_TARGET_' + ($Target.ToUpper() -replace '-', '_') + '_RUSTFLAGS'
  [Environment]::SetEnvironmentVariable($flagVar, '-C target-feature=+crt-static')
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked -p otd --target $Target }
}
$Exe = Join-Path $TargetDir "$Target\release\otd.exe"

# The PE header must match -Arch (Machine), and the app must be a GUI binary (Subsystem 2), so a
# Start menu launch opens no console window.
$bytes = [System.IO.File]::ReadAllBytes($Exe)
$pe = [BitConverter]::ToInt32($bytes, 0x3C)
$machine = [BitConverter]::ToUInt16($bytes, $pe + 4)
$subsystem = [BitConverter]::ToUInt16($bytes, $pe + 0x5C)
$expected = @{ x64 = 0x8664; arm64 = 0xAA64 }[$Arch]
if ($machine -ne $expected) { throw "otd.exe is for machine 0x$('{0:X}' -f $machine), not $Arch" }
if ($subsystem -ne 2) { throw "otd.exe has PE subsystem $subsystem, not 2 (Windows GUI)" }
Write-Output "ok otd.exe: $Arch, Windows GUI subsystem"

$Stage = Join-Path $TargetDir "windows-package\$Arch"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item $Exe, (Join-Path $Root 'NOTICE'), (Join-Path $Root 'ATTRIBUTION.md') $Stage

$Msi = Join-Path $Dist "overthediff-$Version-windows-$Arch.msi"
Invoke-Native 'wix build' {
  wix build (Join-Path $PSScriptRoot 'overthediff.wxs') -arch $Arch -d "Version=$MsiVersion" -d "StageDir=$Stage" -o $Msi
}
# wix writes its debug symbols next to the MSI; keep them out of the release assets.
Remove-Item -Force -ErrorAction SilentlyContinue ([IO.Path]::ChangeExtension($Msi, '.wixpdb'))
Get-Item $Msi | Format-Table Name, Length

if ($env:GITHUB_ACTIONS -eq 'true') {
  & (Join-Path $PSScriptRoot 'smoke.ps1') -Msi $Msi -Version $Version
}
