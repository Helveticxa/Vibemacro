param(
    [string]$IsccPath = ''
)

$ErrorActionPreference = 'Stop'
$project = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Push-Location $project
try {
    cargo fmt -- --check
    if ($LASTEXITCODE -ne 0) { throw 'cargo fmt gagal' }
    cargo clippy --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'cargo clippy gagal' }
    cargo test --all-targets -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'cargo test gagal' }
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

    & $IsccPath (Join-Path $project 'installer\VibeTimer.iss')
    if ($LASTEXITCODE -ne 0) { throw 'Kompilasi installer gagal' }

    $installer = Join-Path $project 'dist\VibeTimer-Setup-1.0.0-x64.exe'
    Get-Item $installer | Select-Object FullName, Length, LastWriteTime
    Get-FileHash $installer -Algorithm SHA256
}
finally {
    Pop-Location
}

