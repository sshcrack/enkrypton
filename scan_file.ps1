Write-Host "Scanning file"
vt scan file src-tauri/target/release/enkrypton.exe | Tee-Object -Variable output | Write-Host

$array = $output -split "\n"
foreach ($item in $array) {
    $maybe_hash = $item -split " "
    $maybe_hash = $maybe_hash[1]

    if (-Not ($maybe_hash -match "^([0-9A-z]|=)+$")) {
        continue
    }

    Write-Host "Hash: $maybe_hash"
    $res = vt analysis file M2QxODdkMDUyMmE0ZTdmYmE5ZDA0M2I5ZmI4YTg4NzI6MTcwODA5ODU2Nw== --format json | ConvertFrom-Json
    Write-Host "Stats: $($res.stats)"
}