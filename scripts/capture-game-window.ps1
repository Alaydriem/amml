# Screenshots the game window of one PID without focus, then posts Ctrl+Shift+F10 to that window only and
# screenshots it again 2 s later. Writes <Out>-before.png and <Out>-hotkey.png.
#
# The capture is PrintWindow with PW_RENDERFULLCONTENT, so the window does not need to be in front. The keys are
# PostMessage WM_KEYDOWN/WM_KEYUP to that one HWND: no SendInput, no SetForegroundWindow. They are an attempt only and
# cannot open the manager panel: the manager reads Ctrl and Shift with GetAsyncKeyState, which posted messages do not
# change, so it sees F10 alone.
#
# Only call it for the PID the ingame-* task recorded and re-checked. Exit 1 when the PID has no visible window.
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][string]$Out
)
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

public static class GameWindow {
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int Left, Top, Right, Bottom; }
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint mapType);
  [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

  // without it GetClientRect gives scaled pixels and PrintWindow's physical-size picture is cut at the right and
  // bottom; -4 is DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
  public static void PhysicalPixels() { SetThreadDpiAwarenessContext((IntPtr)(-4)); }

  const uint PW_CLIENTONLY = 1, PW_RENDERFULLCONTENT = 2;
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

  public static string Shot(IntPtr hwnd, string path) {
    RECT r;
    GetClientRect(hwnd, out r);
    int w = r.Right - r.Left, h = r.Bottom - r.Top;
    if (w <= 0 || h <= 0) return "empty client area";
    using (var bmp = new Bitmap(w, h, PixelFormat.Format32bppArgb)) {
      bool ok;
      using (var g = Graphics.FromImage(bmp)) {
        IntPtr hdc = g.GetHdc();
        ok = PrintWindow(hwnd, hdc, PW_CLIENTONLY | PW_RENDERFULLCONTENT);
        g.ReleaseHdc(hdc);
      }
      bmp.Save(path, ImageFormat.Png);
      return (ok ? "" : "PrintWindow failed; ") + w + "x" + h + " " + path;
    }
  }

  static void Key(IntPtr hwnd, uint vk, bool down) {
    uint scan = MapVirtualKeyW(vk, 0);
    long lParam = 1 | ((long)scan << 16) | (down ? 0 : (3L << 30));
    PostMessageW(hwnd, down ? WM_KEYDOWN : WM_KEYUP, (IntPtr)vk, (IntPtr)lParam);
  }

  // Ctrl+Shift+F10, in press order, released in reverse
  public static void Hotkey(IntPtr hwnd) {
    uint[] keys = { 0x11, 0x10, 0x79 };
    foreach (var k in keys) Key(hwnd, k, true);
    for (int i = keys.Length - 1; i >= 0; i--) Key(hwnd, keys[i], false);
  }
}
'@
[GameWindow]::PhysicalPixels()
$hwnd = [GameWindow]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { "no visible window for pid $ProcessId"; exit 1 }
"window 0x{0:x} of pid {1}" -f $hwnd.ToInt64(), $ProcessId
"screenshot: " + [GameWindow]::Shot($hwnd, "$Out-before.png")
[GameWindow]::Hotkey($hwnd)
"posted Ctrl+Shift+F10 to the window"
Start-Sleep -Seconds 2
"screenshot: " + [GameWindow]::Shot($hwnd, "$Out-hotkey.png")
