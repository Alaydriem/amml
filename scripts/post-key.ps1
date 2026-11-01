# Posts one key press (WM_KEYDOWN then WM_KEYUP) to the game window of one PID, without focus.
#
# The keys are PostMessage to that one HWND: no SendInput, no SetForegroundWindow, so nothing reaches any other
# window. A posted message does not change GetAsyncKeyState, so the key must be a chord without a modifier (the
# levels run seeds F9 as the toggle key). Only call it for the PID the ingame-* task recorded and re-checked.
# Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][int]$Key
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GameKey {
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int Left, Top, Right, Bottom; }
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint mapType);

  const uint WM_KEYDOWN = 0x0100, WM_KEYUP = 0x0101;

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

  public static void Press(IntPtr hwnd, uint vk) {
    uint scan = MapVirtualKeyW(vk, 0);
    long down = 1 | ((long)scan << 16);
    long up = down | (3L << 30);
    PostMessageW(hwnd, WM_KEYDOWN, (IntPtr)vk, (IntPtr)down);
    System.Threading.Thread.Sleep(80);
    PostMessageW(hwnd, WM_KEYUP, (IntPtr)vk, (IntPtr)up);
  }
}
'@
$hwnd = [GameKey]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
[GameKey]::Press($hwnd, [uint32]$Key)
"posted key 0x{0:x} to window 0x{1:x} of pid {2}" -f $Key, $hwnd.ToInt64(), $ProcessId
