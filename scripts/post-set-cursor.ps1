# Posts WM_SETCURSOR for the client area (HTCLIENT, mouse move) to the game window of one PID, without focus.
#
# Windows sends WM_SETCURSOR only for real mouse input over the window; posted mouse messages do not cause one. This
# message is PostMessage to that one HWND, so the real cursor stays where it is and nothing reaches any other window.
# Only call it for the PID the ingame-* task recorded and re-checked. Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GameCursor {
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
$hwnd = [GameCursor]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
# WM_SETCURSOR (0x20): wParam the window, lParam HTCLIENT (1) in the low word and WM_MOUSEMOVE (0x200) in the high word
[GameCursor]::PostMessageW($hwnd, 0x0020, $hwnd, [IntPtr](1 -bor (0x200 -shl 16))) | Out-Null
"posted WM_SETCURSOR to window 0x{0:x} of pid {1}" -f $hwnd.ToInt64(), $ProcessId
