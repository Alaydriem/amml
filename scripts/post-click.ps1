# Posts one left click (mouse move, button down, button up) to the game window of one PID, at a point given in
# fractions of its client area, without focus. Posted mouse messages reach only that window; the game may read the
# mouse another way and ignore them, so a caller checks the effect.
#
# Only call it for the PID the ingame-* task recorded and re-checked. Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][double]$RX,
  [Parameter(Mandatory = $true)][double]$RY
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GameClick {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

  // the largest visible top-level window of the process
  public static IntPtr Find(uint pid) {
    IntPtr best = IntPtr.Zero;
    long bestArea = 0;
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

  public static void Click(IntPtr hwnd, double rx, double ry) {
    RECT r;
    GetClientRect(hwnd, out r);
    int x = (int)((r.Right - r.Left) * rx), y = (int)((r.Bottom - r.Top) * ry);
    IntPtr at = (IntPtr)((y << 16) | (x & 0xFFFF));
    // WM_MOUSEMOVE, WM_LBUTTONDOWN with MK_LBUTTON, WM_LBUTTONUP
    PostMessageW(hwnd, 0x0200, IntPtr.Zero, at);
    System.Threading.Thread.Sleep(100);
    PostMessageW(hwnd, 0x0201, (IntPtr)1, at);
    System.Threading.Thread.Sleep(80);
    PostMessageW(hwnd, 0x0202, IntPtr.Zero, at);
  }
}
'@
# -4 is DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: client coordinates in physical pixels
[GameClick]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$hwnd = [GameClick]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { Write-Host "no visible window for PID $ProcessId"; exit 1 }
[GameClick]::Click($hwnd, $RX, $RY)
Write-Host "posted a click at $RX,$RY to PID $ProcessId"
