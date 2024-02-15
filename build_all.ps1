#!/usr/bin/env pwsh
param (
    [switch]$compileWindows
 )

function Get-CommandExist {
    param (
        [string]$command
    )

    $commandPath = Get-Command $command -ErrorAction SilentlyContinue
    return $commandPath -ne $null
}

function Build-Features {
    param (
        [string]$features,
        [string]$target
    )

    Write-Host "Building with features '$features' and target '$target'"

    $file = "enkrypton"
    $ext = ""
    $cargoOut = "release"
    if($target.Contains("windows")) {
        $ext = ".exe"
        $cargoOut = $target + "/$cargoOut"
    }

    $features_str = $features.Replace("-", "_")
    $features_str = $features_str.Replace(",", "-")
    $features_str = $features_str.Replace("-vendored", "")
    $features_str = $features_str.Replace("vendored", "")

    if($features_str -ne "") {
        $features_str = "-$features_str"
    }

    yarn tauri build -b --features $features -c "./src-tauri/tauri-nobuild.json" $(if($target -ne "") { "--target" }) $(if($target -ne "") { $target })
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    Copy-Item -Path "./src-tauri/target/$cargoOut/$file$ext" -Destination "./build/$file$features_str$ext" -Force
}

if($(-Not $(Get-CommandExist("lld"))) -and $compileWindows) {
    Write-Host "Windows cross-compilation tools not found. Skipping Windows build. https://tauri.app/v1/guides/building/cross-platform/#experimental-build-windows-apps-on-linux-and-macos"
    $compileWindows = $false
}

$config = ConvertFrom-Json $(Get-Content "./src-tauri/tauri.conf.json" -Raw)

Invoke-Expression $($config.build.beforeBuildCommand)
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Remove-Item -r ./build
New-Item -ItemType Directory -Path ./build

$windowsTarget = "x86_64-pc-windows-msvc"

$features = "", "snowflake", "dev", "dev,snowflake"
foreach ($f in $features) {
    Build-Features -features $f
    if($compileWindows) {
        if ($f -eq "") {
            $f = "vendored"
        } else {
            $f = "$f,vendored"
        }

        Build-Features -features $f -target $windowsTarget
    }
}

if($compileWindows) {
    Build-Features -features "enable-console,vendored" -target $windowsTarget
}