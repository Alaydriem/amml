# Posts mouse moves (WM_MOUSEMOVE, no button) across the middle of the game window of one PID, without focus.
#
# The messages are PostMessage to that one HWND: no SendInput, no SetCursorPos, so the real cursor stays where it is
# and nothing reaches any other window. They give the game a mouse feed with a pointer position, so mouse injection
# can bind without a human. The game may read the mouse another way and ignore them, so a caller checks the effect.
# Only call it for the PID the ingame-* task recorded and re-checked. Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GameMouse {
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int Left, Top, Right, Bottom; }
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);

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

  public static void Moves(IntPtr hwnd) {
    RECT r;
    GetClientRect(hwnd, out r);
    int cx = (r.Right - r.Left) / 2, cy = (r.Bottom - r.Top) / 2;
    for (int i = 0; i < 5; i++) {
      long at = ((long)(cy + i) << 16) | (uint)(cx + i * 4);
      // WM_MOUSEMOVE, no button held
      PostMessageW(hwnd, 0x0200, IntPtr.Zero, (IntPtr)at);
      System.Threading.Thread.Sleep(30);
    }
  }
}
'@
$hwnd = [GameMouse]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
[GameMouse]::Moves($hwnd)
"posted mouse moves to window 0x{0:x} of pid {1}" -f $hwnd.ToInt64(), $ProcessId
