# Windows counterpart of make_bundle_zip.sh. Stages the plugin modules in the folders the plugin
# manager installs them to (under C:\Program Files\Common Files), and zips them. The zip root is
# "Common Files\...", so the manager copies each entry under %ProgramFiles% as-is.
#
# RadarVU local copy 2026-11-02: CLAP/LV2 optional (project does not wire them yet). Original usage: pwsh -File make_bundle_zip.ps1 -Plugin "Blue Channel" -Version 1.0.0 -Artefacts <build\..\Release> -Out <dir> [-ResourcesDir <dir>]
# Writes: <Out>\<Plugin>-<Version>-windows-x64-bundles.zip and its .installs.json sidecar.
# -ResourcesDir is copied to "OpenGrid\<Plugin>" under Program Files (licences, manuals).
# Not run yet: needs a real Windows build. Standalone executables are not included.
param(
    [Parameter(Mandatory)][string]$Plugin,
    [Parameter(Mandatory)][string]$Version,
    [Parameter(Mandatory)][string]$Artefacts,
    [Parameter(Mandatory)][string]$Out,
    [string]$MinOs = "10.0",
    [string]$ResourcesDir
)
$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$stage = Join-Path $Out "stage-windows-bundles"
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $stage | Out-Null

# format folder | module name | install folder under Common Files | manifest format
$modules = @(
    @{ Src = "VST3\$Plugin.vst3";        Dest = "Common Files\VST3";                        Format = "vst3" },
    @{ Src = "CLAP\$Plugin.clap";        Dest = "Common Files\CLAP";                        Format = "clap"; Optional = $true },
    @{ Src = "LV2\$Plugin.lv2";          Dest = "Common Files\LV2";                         Format = "lv2"; Optional = $true },
    @{ Src = "AAX\$Plugin.aaxplugin";    Dest = "Common Files\Avid\Audio\Plug-Ins";         Format = "aax"; Optional = $true }
)
if ($ResourcesDir) {
    $resDest = Join-Path $stage "OpenGrid\$Plugin"
    New-Item -ItemType Directory -Force -Path $resDest | Out-Null
    Copy-Item -Recurse -Force (Join-Path $ResourcesDir "*") $resDest
}
$installs = @()
foreach ($m in $modules) {
    $src = Join-Path $Artefacts $m.Src
    if (-not (Test-Path $src)) {
        if ($m.Optional) { continue }
        throw "missing $src"
    }
    $destDir = Join-Path $stage $m.Dest
    New-Item -ItemType Directory -Force -Path $destDir | Out-Null
    Copy-Item -Recurse -Force $src $destDir
    $rel = (Join-Path $m.Dest (Split-Path $src -Leaf)).Replace("\", "/")
    $installs += @{ format = $m.Format; path = $rel }
}

$name = "$Plugin-$Version-windows-x64-bundles.zip" -replace " ", "-"
$zip = Join-Path $Out $name
Remove-Item -Force $zip -ErrorAction SilentlyContinue
$roots = @(Join-Path $stage "Common Files")
if (Test-Path (Join-Path $stage "OpenGrid")) { $roots += Join-Path $stage "OpenGrid" }
Compress-Archive -Path $roots -DestinationPath $zip
Remove-Item -Recurse -Force $stage

$json = @{ os = "windows"; arch = "x64"; min_os = $MinOs; installs = $installs } |
    ConvertTo-Json -Depth 4
# UTF-8 without a BOM, so the manifest writer (and PowerShell 5.1) reads the sidecar cleanly.
[System.IO.File]::WriteAllText("$zip.installs.json", $json, (New-Object System.Text.UTF8Encoding($false)))
Write-Host "wrote $zip"
