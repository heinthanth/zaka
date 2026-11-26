# Install zakac and zakafmt, the Zaka compiler and formatter, on Windows.
# `-Help` lists the options; each has a variable as well, and the option wins.

param(
  [string]$Version = $env:ZAKA_VERSION,
  [string]$Dir = $env:ZAKA_HOME,
  [switch]$NoPath,
  [switch]$Help
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

if ($Help) {
  Write-Host @'
Install zakac and zakafmt, the Zaka compiler and formatter, on Windows.
The executables of the newest release go to %USERPROFILE%\.zaka\bin, which then
joins the PATH of the user.

Usage:
  irm https://zaka-lang.org/install.ps1 | iex
  & ([scriptblock]::Create((irm https://zaka-lang.org/install.ps1))) [options]
  .\install.ps1 [options]

Options:
  -Version <tag>  install the release <tag>, as v0.1.0, in place of the newest
  -Dir <dir>      install into <dir>\bin, in place of %USERPROFILE%\.zaka\bin
  -NoPath         print the folder, and leave the PATH as is
  -Help           print this help

Variables:
  ZAKA_VERSION    as -Version
  ZAKA_HOME       as -Dir
  ZAKA_NO_PATH    as -NoPath, when set
  ZAKA_RELEASES   where the releases are, for tests of this script
'@
  return
}

$releases = if ($env:ZAKA_RELEASES) { $env:ZAKA_RELEASES } else { 'https://github.com/heinthanth/zaka/releases' }
$zakaHome = if ($Dir) { $Dir } else { Join-Path $HOME '.zaka' }
$bin = Join-Path $zakaHome 'bin'
$keepPath = $NoPath -or [bool]$env:ZAKA_NO_PATH

# The CPU of the OS, not of the shell: an x64 PowerShell on ARM64 runs under
# emulation, and the ARM64 build suits the computer.
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$cpu = switch ($arch) {
  'X64' { 'x86_64' }
  'Arm64' { 'aarch64' }
  default { throw "zaka install: no build for the CPU $arch" }
}
$target = "$cpu-pc-windows-msvc"
$asset = "zakac-$target.zip"
$url = if ($Version) { "$releases/download/$Version/$asset" } else { "$releases/latest/download/$asset" }

$work = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
New-Item -ItemType Directory $work | Out-Null
try {
  Write-Host "Downloading $url"
  Invoke-WebRequest $url -OutFile (Join-Path $work $asset) -UseBasicParsing
  Expand-Archive (Join-Path $work $asset) -DestinationPath $work
  New-Item -ItemType Directory -Force $bin | Out-Null
  $unpacked = Join-Path $work "zakac-$target"
  Copy-Item (Join-Path $unpacked 'zakac.exe'), (Join-Path $unpacked 'zakafmt.exe') $bin -Force
}
finally {
  Remove-Item -Recurse -Force $work
}
$version = & (Join-Path $bin 'zakac.exe') --version | Select-Object -First 1
Write-Host "Installed $version in $bin"

# The PATH of the user, which new terminals read.
$path = [Environment]::GetEnvironmentVariable('Path', 'User')
$parts = if ($path) { $path -split ';' } else { @() }
if ($parts -contains $bin) { return }
if ($keepPath) {
  Write-Host "Add $bin to the PATH of the user"
  return
}
[Environment]::SetEnvironmentVariable('Path', (@($bin) + $parts) -join ';', 'User')
Write-Host "Added $bin to the PATH of the user; open a new terminal to use zakac"
