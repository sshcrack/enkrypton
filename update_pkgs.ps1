$configDirs = @("src-packages", "src-tauri") | Get-ChildItem -Exclude target -Recurse -Filter Cargo.toml -Depth 2 | ForEach-Object { $_.Directory.FullName }
Write-Host "Updating index..."
cargo install empty-library 2> $NULL
foreach ($dir in $configDirs) {
    Write-Host "Updating packages in $dir"
    Push-Location $dir
    cargo update
    Pop-Location
}

Write-Host "Dependencies updated."