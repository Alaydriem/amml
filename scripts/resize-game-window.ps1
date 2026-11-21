# Resizes, minimises or restores the game window of one PID, in the style of post-key.ps1: SetWindowPos with
# SWP_NOACTIVATE for a size, ShowWindow for minimise and restore. Prints the client and outer sizes before and after.
#
# Only call it for the PID the ingame-* task recorded and re-checked. Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  # The new outer window size; left out with -Minimise or -Restore
  [int]$Width = 0,
  [int]$Height = 0,
  [switch]$Minimise,
  [switch]$Restore
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GameWindowSize {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int command);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

  // the visible top-level window of the process with the largest client area; a minimised one (0 x 0) counts too
  public static IntPtr Find(uint pid) {
    IntPtr best = IntPtr.Zero;
    long bestArea = -1;
    EnumWindows((hwnd, _) => {
      uint owner;
      GetWindowThreadProcessId(hwnd, out owner);
      RECT r;
      if (owner == pid && IsWindowVisible(hwnd) && GetClientRect(hwnd, out r)) {
        long area = (long)(r.Right - r.Left) * (r.Bottom - r.Top);
        if (area > bestArea) { bestArea = area; best = hwnd; }
      }
      return true;
    }, IntPtr.Zero);
    return best;
  }

  public static string Client(IntPtr hwnd) {
    RECT r;
    GetClientRect(hwnd, out r);
    RECT w;
    GetWindowRect(hwnd, out w);
    return string.Format("{0}x{1}{2} outer={3}x{4}", r.Right - r.Left, r.Bottom - r.Top, IsIconic(hwnd) ? "(minimised)" : "", w.Right - w.Left, w.Bottom - w.Top);
  }
}
'@
# -4 is DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: sizes in physical pixels
[GameWindowSize]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$hwnd = [GameWindowSize]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
$before = [GameWindowSize]::Client($hwnd)
if ($Minimise) {
  # SW_MINIMIZE
  [GameWindowSize]::ShowWindow($hwnd, 6) | Out-Null
} elseif ($Restore) {
  # SW_RESTORE
  [GameWindowSize]::ShowWindow($hwnd, 9) | Out-Null
} elseif ($Width -gt 0 -and $Height -gt 0) {
  # a maximised window keeps its size; restore it first without activating it (SW_SHOWNOACTIVATE)
  if ([GameWindowSize]::IsZoomed($hwnd)) { [GameWindowSize]::ShowWindow($hwnd, 4) | Out-Null }
  # SWP_NOMOVE 0x2 | SWP_NOZORDER 0x4 | SWP_NOACTIVATE 0x10
  [GameWindowSize]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $Width, $Height, 0x16) | Out-Null
} else {
  "give -Width and -Height, -Minimise or -Restore"
  exit 1
}
Start-Sleep -Milliseconds 300
"window 0x{0:x} of pid {1}: client before={2} after={3}" -f $hwnd.ToInt64(), $ProcessId, $before, [GameWindowSize]::Client($hwnd)
