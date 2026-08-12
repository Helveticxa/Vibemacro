param(
    [string]$IsccPath = ''
)

$ErrorActionPreference = 'Stop'
$project = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$cargoToml = Get-Content -LiteralPath (Join-Path $project 'Cargo.toml') -Raw
$versionMatch = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"')
if (-not $versionMatch.Success) { throw 'Versi package tidak ditemukan di Cargo.toml' }
$version = $versionMatch.Groups[1].Value
$qaParent = [System.IO.Path]::GetFullPath((Join-Path $project 'qa'))
$qaRoot = [System.IO.Path]::GetFullPath((Join-Path $qaParent 'installer-upgrade'))
$fixtureDir = Join-Path $qaRoot 'fixtures'
$installDir = Join-Path $qaRoot 'app'
$localData = Join-Path $qaRoot 'localappdata'
$legacyData = Join-Path $localData 'VibeTimer'
$newData = Join-Path $localData 'Vibemacro'
$qaAppId = '{{22C5EF59-EEA3-4F81-BA7B-C3EF16383980}'
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{22C5EF59-EEA3-4F81-BA7B-C3EF16383980}_is1'
$productionKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{F677B6B9-347D-4D9F-9444-23A7DA9C6822}_is1'
$productionBefore = if (Test-Path -LiteralPath $productionKey) {
    (Get-ItemProperty -LiteralPath $productionKey).UninstallString
} else {
    $null
}

if (-not $qaRoot.StartsWith($qaParent + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "QA root tidak aman: $qaRoot"
}
if (Test-Path -LiteralPath $uninstallKey) { throw 'Registrasi QA lama masih ada; bersihkan secara manual' }
if (Test-Path -LiteralPath $qaRoot) { Remove-Item -LiteralPath $qaRoot -Recurse -Force }
New-Item -ItemType Directory -Path $fixtureDir,$legacyData -Force | Out-Null

if ([string]::IsNullOrWhiteSpace($IsccPath)) {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 7\ISCC.exe'),
        (Join-Path $env:ProgramFiles 'Inno Setup 7\ISCC.exe'),
        (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 7\ISCC.exe'),
        (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe')
    )
    $IsccPath = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if ([string]::IsNullOrWhiteSpace($IsccPath)) { throw 'ISCC.exe tidak ditemukan' }

# VTS1 v1 dengan update checks nonaktif agar smoke test tidak menyentuh jaringan.
[byte[]]$settings = 0x56,0x54,0x53,0x31,0x01,0x00,0x01,0x01,0x00,0x00,0x01,0x08,0x07,0x00,0x00,0x10,0x27,0x00,0x00,0x00
[System.IO.File]::WriteAllBytes((Join-Path $legacyData 'settings.vts'), $settings)
$legacyHash = (Get-FileHash (Join-Path $legacyData 'settings.vts') -Algorithm SHA256).Hash

$primary = $null
$previousLocalAppData = $env:LOCALAPPDATA
try {
    & $IsccPath "/DMyAppId=$qaAppId" "/DMyOutputDir=$fixtureDir" (Join-Path $project 'installer\LegacyUpgradeFixture.iss') | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Legacy fixture gagal dibuat' }
    & $IsccPath "/DMyAppId=$qaAppId" "/DMyOutputDir=$fixtureDir" "/DMyOutputBaseFilename=Vibemacro-Setup-$version-QA" (Join-Path $project 'installer\Vibemacro.iss') | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Vibemacro QA installer gagal dibuat' }

    $oldInstaller = Join-Path $fixtureDir 'VibeTimer-Setup-1.0.0-QA.exe'
    $newInstaller = Join-Path $fixtureDir "Vibemacro-Setup-$version-QA.exe"
    $oldSetup = Start-Process -FilePath $oldInstaller -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',"/DIR=$installDir" -WindowStyle Hidden -PassThru -Wait
    if ($oldSetup.ExitCode -ne 0) { throw "Install 1.0 QA gagal: $($oldSetup.ExitCode)" }
    $oldExePresent = Test-Path -LiteralPath (Join-Path $installDir 'VibeTimer.exe')

    $newSetup = Start-Process -FilePath $newInstaller -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',"/DIR=$installDir" -WindowStyle Hidden -PassThru -Wait
    if ($newSetup.ExitCode -ne 0) { throw "Upgrade $version QA gagal: $($newSetup.ExitCode)" }
    $newExe = Join-Path $installDir 'Vibemacro.exe'
    $newExePresent = Test-Path -LiteralPath $newExe
    $oldExeRemoved = -not (Test-Path -LiteralPath (Join-Path $installDir 'VibeTimer.exe'))
    $registration = Get-ItemProperty -LiteralPath $uninstallKey

    $env:LOCALAPPDATA = $localData
    $primary = Start-Process -FilePath $newExe -ArgumentList '--background' -WindowStyle Hidden -PassThru
    Start-Sleep -Milliseconds 1200
    if ($primary.HasExited) { throw "Vibemacro smoke process keluar: $($primary.ExitCode)" }
    $second = Start-Process -FilePath $newExe -ArgumentList '--background' -WindowStyle Hidden -PassThru -Wait
    $singleInstanceExit = $second.ExitCode
    $migrationHash = (Get-FileHash (Join-Path $newData 'settings.vts') -Algorithm SHA256).Hash
    $migrationCopied = $migrationHash -eq $legacyHash
    $legacyPreserved = Test-Path -LiteralPath (Join-Path $legacyData 'settings.vts')

    Stop-Process -Id $primary.Id -Force
    $primary.WaitForExit()
    $primary = $null

    $uninstaller = Join-Path $installDir 'unins000.exe'
    $uninstall = Start-Process -FilePath $uninstaller -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -WindowStyle Hidden -PassThru -Wait
    if ($uninstall.ExitCode -ne 0) { throw "Uninstall QA gagal: $($uninstall.ExitCode)" }
    $installRemoved = -not (Test-Path -LiteralPath $installDir)
    $dataPreserved = (Test-Path -LiteralPath $legacyData) -and (Test-Path -LiteralPath $newData)
    $registrationRemoved = -not (Test-Path -LiteralPath $uninstallKey)
    $productionAfter = if (Test-Path -LiteralPath $productionKey) {
        (Get-ItemProperty -LiteralPath $productionKey).UninstallString
    } else {
        $null
    }

    [pscustomobject]@{
        OldInstallSucceeded = $oldExePresent
        NewInstallSucceeded = $newExePresent
        OldExecutableRemoved = $oldExeRemoved
        DisplayName = $registration.DisplayName
        DisplayVersion = $registration.DisplayVersion
        SingleInstanceExitCode = $singleInstanceExit
        LegacyDataCopiedExactly = $migrationCopied
        LegacyDataPreserved = $legacyPreserved
        UninstallRemovedInstallDirectory = $installRemoved
        UninstallPreservedUserData = $dataPreserved
        UninstallRegistrationRemoved = $registrationRemoved
        ProductionRegistrationUntouched = $productionBefore -eq $productionAfter
    } | Format-List
}
finally {
    $env:LOCALAPPDATA = $previousLocalAppData
    if ($null -ne $primary -and -not $primary.HasExited) {
        Stop-Process -Id $primary.Id -Force -ErrorAction SilentlyContinue
    }
    $remainingUninstaller = Join-Path $installDir 'unins000.exe'
    if (Test-Path -LiteralPath $remainingUninstaller) {
        $cleanupUninstall = Start-Process -FilePath $remainingUninstaller -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -WindowStyle Hidden -PassThru -Wait
    }
    if (Test-Path -LiteralPath $qaRoot) {
        for ($attempt = 0; $attempt -lt 5; $attempt++) {
            try {
                Remove-Item -LiteralPath $qaRoot -Recurse -Force -ErrorAction Stop
                break
            } catch {
                if ($attempt -eq 4) { Write-Warning "Fixture QA belum dapat dibersihkan: $($_.Exception.Message)" }
                Start-Sleep -Milliseconds 500
            }
        }
    }
}
