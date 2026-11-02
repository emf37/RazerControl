param(
    [string]$Out = 'D:\tools\ctf\script\DS_Desktop\aday\razer-driver\shot.png'
)
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class RCShot {
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowW(string c, string n);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint f);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
}
"@
$h = [RCShot]::FindWindowW('Window Class', 'RazerControl')
if ($h -eq [IntPtr]::Zero) { 'NO_WINDOW'; exit 1 }
[RCShot]::SetWindowPos($h, [IntPtr](-2), 0, 0, 0, 0, 0x0043) | Out-Null
[RCShot]::SetWindowPos($h, [IntPtr](0), 0, 0, 0, 0, 0x0043) | Out-Null
[RCShot]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 350
$r = New-Object RCShot+RECT
[RCShot]::GetWindowRect($h, [ref]$r) | Out-Null
$w = $r.R - $r.L
$ht = $r.B - $r.T
if ($w -le 0 -or $ht -le 0) { 'BAD_RECT'; exit 1 }
$bmp = New-Object System.Drawing.Bitmap($w, $ht)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.L, $r.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Write-Output ("SAVED {0} {1}x{2} at {3},{4}" -f $Out, $w, $ht, $r.L, $r.T)
