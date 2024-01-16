function Build-Features {
    param (
        [string]$features
    )

    Write-Host Building with features: $features

    $file = "enkrypton"
    $ext = ""
    if($IsWindows) {
        $ext = ".exe"
    }

    $features_Str = $features.Replace("-", "_")
    $features_str = $features.Replace(",", "-")

    if($features_str -ne "") {
        $features_str = "-$features_str"
    }

    yarn tauri build -b --features $features -c "./src-tauri/tauri-nobuild.json"
    Copy-Item -Path "./src-tauri/target/release/$file$ext" -Destination "./build/$file$features_str$ext" -Force
}

$config = ConvertFrom-Json $(Get-Content "./src-tauri/tauri.conf.json" -Raw)

Invoke-Expression $($config.build.beforeBuildCommand)

Remove-Item -r ./build
New-Item -ItemType Directory -Path ./build

Build-Features -features ""
if($IsWindows) {
    Build-Features -features "enable-console"
}
Build-Features -features "snowflake"
Build-Features -features "dev"
Build-Features -features "dev,snowflake"