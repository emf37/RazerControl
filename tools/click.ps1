param(
    [int]$Dx = 0,
    [int]$Dy = 0,
    [switch]$Drag,
    [int]$DragDx = 0,
    [int]$DragDy = 0
)
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class RCWin {
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowW(string c, string n);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint f);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
}
"@
$h = [RCWin]::FindWindowW('Window Class', 'RazerControl')
if ($h -eq [IntPtr]::Zero) { 'NO_WINDOW'; exit 1 }
$r = New-Object RCWin+RECT
[RCWin]::GetWindowRect($h, [ref]$r) | Out-Null
# 先解除「总在最前」，再抬到普通窗口层顶部（否则会把窗口永久置顶）
[RCWin]::SetWindowPos($h, [IntPtr](-2), 0, 0, 0, 0, 0x0043) | Out-Null
[RCWin]::SetWindowPos($h, [IntPtr](0), $r.L, $r.T, $r.R - $r.L, $r.B - $r.T, 0x0040) | Out-Null
[RCWin]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 200
$x = $r.L + $Dx
$y = $r.T + $Dy
[RCWin]::SetCursorPos($x, $y) | Out-Null
Start-Sleep -Milliseconds 90
if ($Drag) {
    [RCWin]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    for ($i = 1; $i -le 10; $i++) {
        [RCWin]::SetCursorPos($x + [int]($DragDx * $i / 10), $y + [int]($DragDy * $i / 10)) | Out-Null
        Start-Sleep -Milliseconds 25
    }
    [RCWin]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
} else {
    [RCWin]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 60
    [RCWin]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
}
Start-Sleep -Milliseconds 280
Write-Output ("clicked {0},{1} (window {2},{3})" -f $x, $y, $r.L, $r.T)
