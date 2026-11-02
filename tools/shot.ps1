param(
    [string]$Exe = 'D:\tools\ctf\script\DS_Desktop\aday\razer-driver\target\release\RazerControl.exe',
    [string]$Out = 'D:\tools\ctf\script\DS_Desktop\aday\razer-driver\shot.png',
    [int]$Wait = 0
)
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string cls, string name);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
}
"@
if ($Wait -gt 0) { Start-Sleep -Milliseconds $Wait }
$h = [W]::FindWindow($null, 'RazerControl')
if ($h -eq [IntPtr]::Zero) {
    $p = Get-Process RazerControl -ErrorAction SilentlyContinue
    if ($p) { $h = $p[0].MainWindowHandle }
}
if ($h -eq [IntPtr]::Zero) { Write-Output 'WINDOW_NOT_FOUND'; exit 1 }
[W]::ShowWindow($h, 5) | Out-Null
[W]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 450
$r = New-Object W+RECT
[W]::GetWindowRect($h, [ref]$r) | Out-Null
$w = $r.R - $r.L; $ht = $r.B - $r.T
if ($w -le 0 -or $ht -le 0) { Write-Output 'BAD_RECT'; exit 1 }
$bmp = New-Object System.Drawing.Bitmap($w, $ht)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.L, $r.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Write-Output ("SAVED {0} {1}x{2} at {3},{4}" -f $Out, $w, $ht, $r.L, $r.T)
