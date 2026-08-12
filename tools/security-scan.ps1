param(
    [switch]$IncludeHistory
)

$ErrorActionPreference = 'Stop'
$project = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$self = [System.IO.Path]::GetFullPath($PSCommandPath)

$rules = @(
    @{ Name = 'private key'; Pattern = '-----BEGIN (RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----' },
    @{ Name = 'GitHub token'; Pattern = ('gh' + '[pousr]_[A-Za-z0-9]{20,}') },
    @{ Name = 'GitHub fine-grained token'; Pattern = ('github_pat_' + '[A-Za-z0-9_]{20,}') },
    @{ Name = 'AWS access key'; Pattern = ('AKIA' + '[A-Z0-9]{16}') },
    @{ Name = 'Google API key'; Pattern = ('AIza' + '[A-Za-z0-9_-]{30,}') },
    @{ Name = 'Slack token'; Pattern = ('xox' + '[abprs]-[A-Za-z0-9-]{20,}') },
    @{ Name = 'generic assigned secret'; Pattern = '(?i)(api[_-]?key|client[_-]?secret|access[_-]?token|password)\s*[:=]\s*["''][^"'']{12,}["'']' }
)

function Test-Text([string]$Text, [string]$Location) {
    $findings = @()
    foreach ($rule in $rules) {
        if ([regex]::IsMatch($Text, $rule.Pattern)) {
            $findings += "$Location [$($rule.Name)]"
        }
    }
    return $findings
}

$findings = @()
$tracked = git -C $project ls-files --cached --others --exclude-standard
if ($LASTEXITCODE -ne 0) { throw 'git ls-files gagal' }
foreach ($relative in $tracked) {
    $path = [System.IO.Path]::GetFullPath((Join-Path $project $relative))
    if ($path -eq $self -or -not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
    $bytes = [System.IO.File]::ReadAllBytes($path)
    if ($bytes -contains 0) { continue }
    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    $findings += Test-Text $text $relative
}

if ($IncludeHistory) {
    $history = (git -C $project log --all --no-color -p -- . ':!tools/security-scan.ps1') -join "`n"
    if ($LASTEXITCODE -ne 0) { throw 'git history scan gagal' }
    $findings += Test-Text $history 'git history'
}

if ($findings.Count -gt 0) {
    $findings | Sort-Object -Unique | ForEach-Object { Write-Error "Potensi secret: $_" }
    throw 'Security scan menemukan pola credential. Nilai rahasia tidak dicetak.'
}

Write-Output "Security scan lulus: $($tracked.Count) tracked files, tidak ada credential pattern."
if ($IncludeHistory) { Write-Output 'Git history juga dipindai tanpa mencetak nilai yang cocok.' }
