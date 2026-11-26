# Install zakac and zakafmt on Windows, in PowerShell:
#
#   irm https://zaka-lang.org/install.ps1 | iex
#
# The executables of the newest release go to %USERPROFILE%\.zaka\bin, which
# then joins the PATH of the user. These variables change that:
#
#   ZAKA_VERSION   a release to install in place of the newest, as v0.1.0
#   ZAKA_HOME      the folder to install into, in place of %USERPROFILE%\.zaka
#   ZAKA_NO_PATH   when set, the PATH is left as it is
#   ZAKA_RELEASES  where the releases are, for tests of this script

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$releases = if ($env:ZAKA_RELEASES) { $env:ZAKA_RELEASES } else { 'https://github.com/heinthanth/zaka/releases' }
$zakaHome = if ($env:ZAKA_HOME) { $env:ZAKA_HOME } else { Join-Path $HOME '.zaka' }
$bin = Join-Path $zakaHome 'bin'

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
$url = if ($env:ZAKA_VERSION) { "$releases/download/$($env:ZAKA_VERSION)/$asset" } else { "$releases/latest/download/$asset" }

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
if ($env:ZAKA_NO_PATH) {
  Write-Host "Add $bin to the PATH of the user"
  return
}
[Environment]::SetEnvironmentVariable('Path', (@($bin) + $parts) -join ';', 'User')
Write-Host "Added $bin to the PATH of the user; open a new terminal to use zakac"
