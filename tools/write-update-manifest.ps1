param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^https://')]
    [string]$InstallerUrl,
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9]+\.[0-9]+\.[0-9]+$')]
    [string]$Version,
    [string]$InstallerPath = (Join-Path $PSScriptRoot "..\dist\Vibemacro-Setup-$Version-x64.exe"),
    [string]$OutputPath = (Join-Path $PSScriptRoot '..\dist\vibemacro-update.txt')
)

$ErrorActionPreference = 'Stop'
$installer = [System.IO.Path]::GetFullPath($InstallerPath)
if (-not (Test-Path $installer -PathType Leaf)) {
    throw "Installer tidak ditemukan: $installer"
}
$hash = (Get-FileHash $installer -Algorithm SHA256).Hash
$manifest = "VIBEMACRO-UPDATE-1`nversion=$Version`ninstaller=$InstallerUrl`nsha256=$hash`n"
$encoding = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText([System.IO.Path]::GetFullPath($OutputPath), $manifest, $encoding)
Get-Item $OutputPath | Select-Object FullName, Length, LastWriteTime
Write-Output "SHA256=$hash"
