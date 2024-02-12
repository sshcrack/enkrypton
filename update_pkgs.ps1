$configDirs = @("src-packages", "src-tauri") | Get-ChildItem -Exclude target -Recurse -Filter Cargo.toml -Depth 2 | ForEach-Object { $_.Directory.FullName }
foreach ($dir in $configDirs) {
    Write-Host "Updating packages in $dir"
    Push-Location $dir
    cargo upgrade -i
    cargo update
    Pop-Location
}