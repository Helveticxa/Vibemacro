param(
    [string]$IsccPath = '',
    [switch]$SkipValidation,
    [switch]$SkipDesktopE2E
)

$ErrorActionPreference = 'Stop'
$project = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$cargoToml = Get-Content -LiteralPath (Join-Path $project 'Cargo.toml') -Raw
$versionMatch = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"')
if (-not $versionMatch.Success) { throw 'Versi package tidak ditemukan di Cargo.toml' }
$version = $versionMatch.Groups[1].Value
Push-Location $project
try {
    if (-not $SkipValidation) {
        cargo fmt -- --check
        if ($LASTEXITCODE -ne 0) { throw 'cargo fmt gagal' }
        cargo clippy --all-targets -- -D warnings
        if ($LASTEXITCODE -ne 0) { throw 'cargo clippy gagal' }
        if ($SkipDesktopE2E) {
            cargo test --all-targets -- --test-threads=1
        } else {
            cargo test --all-targets -- --include-ignored --test-threads=1
        }
        if ($LASTEXITCODE -ne 0) { throw 'cargo test gagal' }
    }
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw 'cargo build --release gagal' }

    & (Join-Path $PSScriptRoot 'generate-icon.ps1')

    if ([string]::IsNullOrWhiteSpace($IsccPath)) {
        $candidates = @(
            (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 7\ISCC.exe'),
            (Join-Path $env:ProgramFiles 'Inno Setup 7\ISCC.exe'),
            (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 7\ISCC.exe'),
            (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe'),
            (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe')
        )
        $IsccPath = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    }
    if ([string]::IsNullOrWhiteSpace($IsccPath) -or -not (Test-Path $IsccPath)) {
        throw 'ISCC.exe tidak ditemukan. Instal Inno Setup 7 atau isi -IsccPath.'
    }

    & $IsccPath (Join-Path $project 'installer\Vibemacro.iss')
    if ($LASTEXITCODE -ne 0) { throw 'Kompilasi installer gagal' }

    $installer = Join-Path $project "dist\Vibemacro-Setup-$version-x64.exe"
    $portable = Join-Path $project "dist\Vibemacro-$version-portable.exe"
    Copy-Item -LiteralPath (Join-Path $project 'target\release\Vibemacro.exe') -Destination $portable -Force
    $installerUrl = "https://github.com/Helveticxa/Vibemacro/releases/download/v$version/Vibemacro-Setup-$version-x64.exe"
    & (Join-Path $PSScriptRoot 'write-update-manifest.ps1') `
        -Version $version `
        -InstallerUrl $installerUrl `
        -InstallerPath $installer
    if ($LASTEXITCODE -ne 0) { throw 'Pembuatan manifest update gagal' }

    $installerHash = (Get-FileHash $installer -Algorithm SHA256).Hash
    $portableHash = (Get-FileHash $portable -Algorithm SHA256).Hash
    $manifestPath = Join-Path $project 'dist\vibemacro-update.txt'
    $manifestHash = (Get-FileHash $manifestPath -Algorithm SHA256).Hash
    $checksums = "$installerHash  $(Split-Path $installer -Leaf)`n$portableHash  $(Split-Path $portable -Leaf)`n$manifestHash  $(Split-Path $manifestPath -Leaf)`n"
    [System.IO.File]::WriteAllText(
        (Join-Path $project 'dist\SHA256SUMS.txt'),
        $checksums,
        (New-Object System.Text.UTF8Encoding $false)
    )
    Get-Item $installer | Select-Object FullName, Length, LastWriteTime
    Get-FileHash $installer -Algorithm SHA256
}
finally {
    Pop-Location
}
