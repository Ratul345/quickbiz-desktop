<#
  Builds both Windows packages from one source tree.

    .\build.ps1 -Target Installer     # website .exe installer (needs Inno Setup 6)
    .\build.ps1 -Target Msix -IdentityName ... -Publisher ... -PublisherDisplayName ...
                                      # Store package (needs the Windows 10/11 SDK)
    .\build.ps1 -Target All ...

  The MSIX is left UNSIGNED: upload it to Partner Center and Microsoft signs it.
  Pass -SignPfx / -SignPassword only to test-install the package on your own PC.
#>
param(
  [ValidateSet('Installer', 'Msix', 'All')] [string]$Target = 'All',
  [string]$IdentityName = 'InfoSparkLabs.QuickBiz',
  [string]$Publisher = 'CN=00000000-0000-0000-0000-000000000000',
  [string]$PublisherDisplayName = 'InfoSpark Labs',
  [string]$SignPfx,
  [string]$SignPassword
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null

$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
Write-Host "QuickBiz version $version"

function Find-Tool([string]$name, [string[]]$globs) {
  $cmd = Get-Command $name -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }
  # The Windows SDK may be on any drive: ask Windows where it put it.
  $kits = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots' -ErrorAction SilentlyContinue).KitsRoot10
  if ($kits) { $globs += (Join-Path $kits "bin\*\x64\$name.exe") }
  foreach ($g in $globs) {
    $hit = Get-ChildItem $g -ErrorAction SilentlyContinue | Sort-Object FullName -Descending | Select-Object -First 1
    if ($hit) { return $hit.FullName }
  }
  throw "$name was not found. Install it first (see packaging/README.md)."
}

function Build-Exe([string[]]$cargoArgs, [string]$out) {
  Push-Location $root
  try {
    & cargo build --release @cargoArgs
    if ($LASTEXITCODE) { throw 'cargo build failed' }
  } finally { Pop-Location }
  Copy-Item (Join-Path $root 'target\release\quickbiz-desktop.exe') $out -Force
}

if ($Target -in 'Installer', 'All') {
  $iscc = Find-Tool 'iscc' @('D:\Tools\Inno Setup 6\ISCC.exe', 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe', "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe")
  $exe = Join-Path $dist 'QuickBiz-website.exe'
  Build-Exe @() $exe
  & $iscc "/DAppVersion=$version" "/DExePath=$exe" (Join-Path $PSScriptRoot 'quickbiz.iss')
  if ($LASTEXITCODE) { throw 'Inno Setup failed' }
  $setup = Join-Path $dist "QuickBiz-Setup-$version.exe"
  $hash = (Get-FileHash $setup -Algorithm SHA256).Hash.ToLower()
  Write-Host "Installer: $setup"
  Write-Host "SHA-256:   $hash"
}

if ($Target -in 'Msix', 'All') {
  $makeappx = Find-Tool 'makeappx' @('C:\Program Files (x86)\Windows Kits\10\bin\*\x64\makeappx.exe')
  $stage = Join-Path $dist 'msix-stage'
  if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
  New-Item -ItemType Directory -Force (Join-Path $stage 'Assets') | Out-Null

  Build-Exe @('--features', 'store') (Join-Path $stage 'QuickBiz.exe')

  # Package logos are rendered from assets/images/quickbiz_logo.svg by render-logos.ps1 (run it after changing the logo).
  $logos = Join-Path $PSScriptRoot 'msix\Assets'
  foreach ($f in 'StoreLogo.png', 'Square44x44Logo.png', 'Square150x150Logo.png', 'Wide310x150Logo.png') {
    if (-not (Test-Path (Join-Path $logos $f))) { throw "$f is missing. Run packaging\render-logos.ps1 first." }
    Copy-Item (Join-Path $logos $f) (Join-Path $stage 'Assets') -Force
  }

  # Store versions are x.y.z.0 (the Store reserves the last part).
  $parts = @(($version -split '[-+]')[0].Split('.'))
  while ($parts.Count -lt 3) { $parts += '0' }
  $msixVersion = "$($parts[0]).$($parts[1]).$($parts[2]).0"
  (Get-Content (Join-Path $PSScriptRoot 'msix\AppxManifest.xml') -Raw).
    Replace('@IDENTITY_NAME@', $IdentityName).
    Replace('@PUBLISHER@', $Publisher).
    Replace('@PUBLISHER_DISPLAY@', $PublisherDisplayName).
    Replace('@VERSION@', $msixVersion) |
    Set-Content (Join-Path $stage 'AppxManifest.xml') -Encoding UTF8

  $msix = Join-Path $dist "QuickBiz-$msixVersion.msix"
  if (Test-Path $msix) { Remove-Item $msix -Force }
  & $makeappx pack /d $stage /p $msix /o
  if ($LASTEXITCODE) { throw 'makeappx failed' }

  if ($SignPfx) {
    $signtool = Find-Tool 'signtool' @('C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe')
    & $signtool sign /fd SHA256 /f $SignPfx /p $SignPassword $msix
    if ($LASTEXITCODE) { throw 'signtool failed' }
  }
  Write-Host "MSIX: $msix"
}
