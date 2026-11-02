Add-Type @"
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class E {
    public delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr l);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
    public static List<string> List(uint want) {
        var res = new List<string>();
        EnumWindows((h, l) => {
            uint pid; GetWindowThreadProcessId(h, out pid);
            if (pid == want) {
                var t = new StringBuilder(256); GetWindowText(h, t, 256);
                var c = new StringBuilder(256); GetClassName(h, c, 256);
                RECT r; GetWindowRect(h, out r);
                res.Add(string.Format("hwnd={0} vis={1} rect={2},{3} {4}x{5} class='{6}' title='{7}'",
                    h, IsWindowVisible(h), r.L, r.T, r.R-r.L, r.B-r.T, c, t));
            }
            return true;
        }, IntPtr.Zero);
        return res;
    }
}
"@
$p = Get-Process RazerControl -ErrorAction SilentlyContinue
if (-not $p) { 'NO_PROCESS'; exit }
foreach ($line in [E]::List([uint32]$p[0].Id)) { $line }
