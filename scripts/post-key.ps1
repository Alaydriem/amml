# Posts one key press (WM_KEYDOWN then WM_KEYUP) to the game window of one PID, without focus. -Phase down posts the
# WM_KEYDOWN only (the key stays held for the game until a later -Phase up), -Phase up the WM_KEYUP only.
#
# The keys are PostMessage to that one HWND: no SendInput, no SetForegroundWindow, so nothing reaches any other
# window. A posted message does not change GetAsyncKeyState, so the key must be a chord without a modifier (the
# levels run seeds F9 as the toggle key). Only call it for the PID the ingame-* task recorded and re-checked.
# Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][int]$Key,
  [ValidateSet('press', 'down', 'up')][string]$Phase = 'press'
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
    Down(hwnd, vk);
    System.Threading.Thread.Sleep(80);
    Up(hwnd, vk);
  }

  static long DownParam(uint vk) { return 1 | ((long)MapVirtualKeyW(vk, 0) << 16); }

  public static void Down(IntPtr hwnd, uint vk) {
    PostMessageW(hwnd, WM_KEYDOWN, (IntPtr)vk, (IntPtr)DownParam(vk));
  }

  // previous state down, transition up
  public static void Up(IntPtr hwnd, uint vk) {
    PostMessageW(hwnd, WM_KEYUP, (IntPtr)vk, (IntPtr)(DownParam(vk) | (3L << 30)));
  }
}
'@
$hwnd = [GameKey]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
switch ($Phase) {
  'press' { [GameKey]::Press($hwnd, [uint32]$Key) }
  'down' { [GameKey]::Down($hwnd, [uint32]$Key) }
  'up' { [GameKey]::Up($hwnd, [uint32]$Key) }
}
"posted key 0x{0:x} ({1}) to window 0x{2:x} of pid {3}" -f $Key, $Phase, $hwnd.ToInt64(), $ProcessId
