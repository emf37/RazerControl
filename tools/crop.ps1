param(
    [int]$X = 250, [int]$Y = 240, [int]$W = 200, [int]$H = 180, [double]$Zoom = 3.0,
    [string]$Out = 'D:\tools\ctf\script\DS_Desktop\aday\razer-driver\crop.png'
)
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class C {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
}
"@
$p = Get-Process RazerControl -ErrorAction SilentlyContinue
if (-not $p) { 'NO_PROCESS'; exit 1 }
$hwnd = $p[0].MainWindowHandle
[C]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 300
$r = New-Object C+RECT
[C]::GetWindowRect($hwnd, [ref]$r) | Out-Null
$bmp = New-Object System.Drawing.Bitmap($W, $H)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.L + $X, $r.T + $Y, 0, 0, (New-Object System.Drawing.Size($W, $H)))
$g.Dispose()
$big = New-Object System.Drawing.Bitmap([int]($W * $Zoom), [int]($H * $Zoom))
$g2 = [System.Drawing.Graphics]::FromImage($big)
$g2.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g2.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
$g2.DrawImage($bmp, 0, 0, [int]($W * $Zoom), [int]($H * $Zoom))
$g2.Dispose()
$big.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose(); $big.Dispose()
Write-Output "SAVED $Out"
