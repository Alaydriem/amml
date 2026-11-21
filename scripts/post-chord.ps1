# Posts one key chord to the game window of one PID, without focus: WM_KEYDOWN for each key in the order given, then
# WM_KEYUP for each key in reverse order.
#
# The keys are PostMessage to that one HWND: no SendInput, no SetForegroundWindow, so nothing reaches any other
# window. A posted message does not change GetAsyncKeyState; a reader of the chord must track the modifiers from the
# key messages. Only call it for the PID the ingame-* task recorded and re-checked. Exit 1 when the PID has no visible
# window or no key is given.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  # virtual-key codes, comma separated, in press order (for example 17,16,121 for Ctrl+Shift+F10)
  [Parameter(Mandatory = $true)][string]$Keys
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GameChord {
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

  static void Key(IntPtr hwnd, uint vk, bool down) {
    uint scan = MapVirtualKeyW(vk, 0);
    long lParam = 1 | ((long)scan << 16) | (down ? 0 : (3L << 30));
    PostMessageW(hwnd, down ? WM_KEYDOWN : WM_KEYUP, (IntPtr)vk, (IntPtr)lParam);
  }

  public static void Chord(IntPtr hwnd, int[] keys) {
    foreach (int k in keys) {
      Key(hwnd, (uint)k, true);
      System.Threading.Thread.Sleep(40);
    }
    System.Threading.Thread.Sleep(80);
    for (int i = keys.Length - 1; i >= 0; i--) {
      Key(hwnd, (uint)keys[i], false);
      System.Threading.Thread.Sleep(40);
    }
  }
}
'@
$codes = @($Keys -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' } | ForEach-Object { [int]$_ })
if ($codes.Count -eq 0) { "no key given"; exit 1 }
$hwnd = [GameChord]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
[GameChord]::Chord($hwnd, [int[]]$codes)
"posted chord {0} to window 0x{1:x} of pid {2}" -f (($codes | ForEach-Object { "0x{0:x}" -f $_ }) -join "+"), $hwnd.ToInt64(), $ProcessId
