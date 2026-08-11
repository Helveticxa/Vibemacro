param(
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\assets')
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$output = [System.IO.Path]::GetFullPath($OutputDirectory)
[System.IO.Directory]::CreateDirectory($output) | Out-Null
$pngPath = Join-Path $output 'VibeTimer.png'
$icoPath = Join-Path $output 'VibeTimer.ico'

$bitmap = New-Object System.Drawing.Bitmap 256, 256, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$graphics.Clear([System.Drawing.Color]::Transparent)

function New-RoundedPath([System.Drawing.RectangleF]$Rect, [float]$Radius) {
    $diameter = $Radius * 2
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($Rect.X, $Rect.Y, $diameter, $diameter, 180, 90)
    $path.AddArc($Rect.Right - $diameter, $Rect.Y, $diameter, $diameter, 270, 90)
    $path.AddArc($Rect.Right - $diameter, $Rect.Bottom - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($Rect.X, $Rect.Bottom - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()
    return $path
}

$lime = [System.Drawing.Color]::FromArgb(255, 184, 255, 31)
$ink = [System.Drawing.Color]::FromArgb(255, 10, 14, 11)
$edge = [System.Drawing.Color]::FromArgb(255, 220, 255, 128)
$tile = New-RoundedPath (New-Object System.Drawing.RectangleF 14, 14, 228, 228) 52
$tileBrush = New-Object System.Drawing.SolidBrush $lime
$edgePen = New-Object System.Drawing.Pen $edge, 5
$graphics.FillPath($tileBrush, $tile)
$graphics.DrawPath($edgePen, $tile)

$clockPen = New-Object System.Drawing.Pen $ink, 15
$clockPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$clockPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
$graphics.DrawEllipse($clockPen, 68, 68, 120, 120)
$graphics.DrawLine($clockPen, 128, 94, 128, 132)
$graphics.DrawLine($clockPen, 128, 132, 156, 149)
$centerBrush = New-Object System.Drawing.SolidBrush $ink
$graphics.FillEllipse($centerBrush, 119, 123, 18, 18)

$memory = New-Object System.IO.MemoryStream
$bitmap.Save($memory, [System.Drawing.Imaging.ImageFormat]::Png)
$pngBytes = $memory.ToArray()
[System.IO.File]::WriteAllBytes($pngPath, $pngBytes)

$file = [System.IO.File]::Create($icoPath)
$writer = New-Object System.IO.BinaryWriter $file
$writer.Write([UInt16]0)
$writer.Write([UInt16]1)
$writer.Write([UInt16]1)
$writer.Write([Byte]0)
$writer.Write([Byte]0)
$writer.Write([Byte]0)
$writer.Write([Byte]0)
$writer.Write([UInt16]1)
$writer.Write([UInt16]32)
$writer.Write([UInt32]$pngBytes.Length)
$writer.Write([UInt32]22)
$writer.Write($pngBytes)
$writer.Dispose()

$memory.Dispose()
$centerBrush.Dispose()
$clockPen.Dispose()
$edgePen.Dispose()
$tileBrush.Dispose()
$tile.Dispose()
$graphics.Dispose()
$bitmap.Dispose()

Write-Output "Generated $pngPath"
Write-Output "Generated $icoPath"

