param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^https://')]
    [string]$InstallerUrl,
    [string]$InstallerPath = (Join-Path $PSScriptRoot '..\dist\VibeTimer-Setup-1.0.0-x64.exe'),
    [string]$OutputPath = (Join-Path $PSScriptRoot '..\dist\vibetimer-update.txt')
)

$ErrorActionPreference = 'Stop'
$installer = [System.IO.Path]::GetFullPath($InstallerPath)
if (-not (Test-Path $installer -PathType Leaf)) {
    throw "Installer tidak ditemukan: $installer"
}
$hash = (Get-FileHash $installer -Algorithm SHA256).Hash
$manifest = "VIBETIMER-UPDATE-1`nversion=1.0.0`ninstaller=$InstallerUrl`nsha256=$hash`n"
$encoding = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText([System.IO.Path]::GetFullPath($OutputPath), $manifest, $encoding)
Get-Item $OutputPath | Select-Object FullName, Length, LastWriteTime
Write-Output "SHA256=$hash"

