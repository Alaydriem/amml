# Posts one WM_CHAR to the game window of one PID, without focus. The message is PostMessage to that one HWND, so
# nothing reaches any other window.
#
# Only call it for the PID the ingame-* task recorded and re-checked. Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][char]$Char
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GameChar {
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
$hwnd = [GameChar]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
# WM_CHAR with a repeat count of 1
[GameChar]::PostMessageW($hwnd, 0x0102, [IntPtr][int]$Char, [IntPtr]1) | Out-Null
"posted WM_CHAR '{0}' to window 0x{1:x} of pid {2}" -f $Char, $hwnd.ToInt64(), $ProcessId
