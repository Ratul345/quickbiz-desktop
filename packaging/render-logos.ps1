<#
  Renders every Store and MSIX image from the one source logo (assets/images/quickbiz_logo.svg)
  using headless Chrome or Edge, so every size stays sharp and nothing is drawn by hand.

    .\render-logos.ps1

  Writes:
    packaging\msix\Assets\*.png   (used inside the MSIX package by build.ps1)
    packaging\store\images\*.png  (uploaded by hand in Partner Center > Store listing)
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$svgPath = Join-Path $root 'assets\images\quickbiz_logo.svg'
$msixOut = Join-Path $PSScriptRoot 'msix\Assets'
$storeOut = Join-Path $PSScriptRoot 'store\images'
$work = Join-Path $env:TEMP 'qb-logo-render'
New-Item -ItemType Directory -Force $msixOut, $storeOut, $work | Out-Null

$browser = @(
  'C:\Program Files\Google\Chrome\Application\chrome.exe',
  'C:\Program Files (x86)\Google\Chrome\Application\chrome.exe',
  'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe',
  'C:\Program Files\Microsoft\Edge\Application\msedge.exe'
) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $browser) { throw 'Chrome or Edge is needed to render the logo.' }

$svg = (Get-Content $svgPath -Raw)
$brand = '#2D62ED'

function Render([string]$dir, [string]$name, [int]$w, [int]$h, [string]$body, [string]$bg = 'transparent') {
  $html = Join-Path $work "$name.html"
  @"
<!doctype html><meta charset="utf-8"><style>
html,body{margin:0;width:${w}px;height:${h}px;overflow:hidden;background:$bg}
body{display:flex;flex-direction:column;align-items:center;justify-content:center;font-family:'Segoe UI',sans-serif}
svg{display:block}
</style><body>$body</body>
"@ | Set-Content $html -Encoding UTF8
  $png = Join-Path $dir $name
  if (Test-Path $png) { Remove-Item $png -Force }
  # Own temporary profile, so a Chrome that is already open is never reused or disturbed.
  # Paths are quoted because the project folder contains spaces.
  $cli = @('--headless=new', '--disable-gpu', '--hide-scrollbars', '--force-device-scale-factor=1',
    "`"--user-data-dir=$(Join-Path $work 'profile')`"", '--default-background-color=00000000',
    "--window-size=$w,$h", "`"--screenshot=$png`"", ('file:///' + ($html -replace '\\', '/')))
  Start-Process -FilePath $browser -ArgumentList $cli -Wait -NoNewWindow -RedirectStandardError (Join-Path $work 'chrome.log')
  if (-not (Test-Path $png)) { throw "Rendering $name failed (see $(Join-Path $work 'chrome.log'))" }
}

function Square([int]$size) { return ($svg -replace '<svg ', "<svg width=`"$size`" height=`"$size`" ") }

# Plain icon tiles (transparent corners)
foreach ($s in 44, 50, 150) { Render $msixOut "icon-$s.png" $s $s (Square $s) }
Move-Item (Join-Path $msixOut 'icon-44.png') (Join-Path $msixOut 'Square44x44Logo.png') -Force
Move-Item (Join-Path $msixOut 'icon-50.png') (Join-Path $msixOut 'StoreLogo.png') -Force
Move-Item (Join-Path $msixOut 'icon-150.png') (Join-Path $msixOut 'Square150x150Logo.png') -Force
# Wide tile: icon centred on the brand colour
Render $msixOut 'Wide310x150Logo.png' 310 150 (Square 110) $brand
# Store listing: app tile icons
foreach ($s in 300, 150, 71) { Render $storeOut "tile-${s}x${s}.png" $s $s (Square $s) }

# Poster (9:16) and box art (1:1): dark-to-brand gradient with the icon and name
$grad = 'linear-gradient(160deg,#1A43BF 0%,#2D62ED 100%)'
function Art([string]$name, [int]$w, [int]$h, [double]$iconFrac) {
  $icon = [int]($w * $iconFrac)
  $word = [int]($w * 0.085)
  $body = "$(Square $icon)<div style=`"margin-top:$([int]($w*0.05))px;color:#fff;font-weight:700;font-size:${word}px;letter-spacing:-1px`">QuickBiz</div>"
  Render $storeOut $name $w $h $body $grad
}
Art 'poster-720x1080.png' 720 1080 0.5
Art 'poster-1440x2160.png' 1440 2160 0.5
Art 'boxart-1080x1080.png' 1080 1080 0.42
Art 'boxart-2160x2160.png' 2160 2160 0.42

Add-Type -AssemblyName System.Drawing
foreach ($d in $msixOut, $storeOut) {
  Get-ChildItem $d -Filter *.png | ForEach-Object {
    $i = [System.Drawing.Image]::FromFile($_.FullName)
    '{0,-28} {1}x{2}' -f $_.Name, $i.Width, $i.Height
    $i.Dispose()
  }
}
