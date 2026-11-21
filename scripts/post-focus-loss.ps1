# Posts WM_ACTIVATEAPP with wParam 0 (the app lost activation) to the game window of one PID, without
# changing the real focus.
#
# The message is PostMessage to that one HWND: no SetForegroundWindow, so nothing reaches any other window. The game
# may take focus loss from another path and ignore it, so a caller checks the effect. Only call it for the PID the
# ingame-* task recorded and re-checked. Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GameFocus {
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int Left, Top, Right, Bottom; }
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);

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
}
'@
$hwnd = [GameFocus]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
# WM_ACTIVATEAPP, wParam 0: deactivated; lParam 0: no thread id of the window that is activated
[GameFocus]::PostMessageW($hwnd, 0x001C, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
"posted WM_ACTIVATEAPP 0 to window 0x{0:x} of pid {1}" -f $hwnd.ToInt64(), $ProcessId
