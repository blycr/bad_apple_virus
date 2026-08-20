Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class P3 {
    public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    public static int VisibleCount(uint target) {
        int n = 0;
        EnumWindows((h, l) => {
            uint pid; GetWindowThreadProcessId(h, out pid);
            if (pid == target && IsWindowVisible(h)) n++;
            return true;
        }, IntPtr.Zero);
        return n;
    }
}
'@
[P3]::SetProcessDPIAware() | Out-Null
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$p = Start-Process -FilePath '.\target\release\bad_apple_win.exe' -WorkingDirectory (Get-Location) -PassThru
$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$prev = 0
foreach ($t in 6, 12, 20) {
    Start-Sleep -Seconds ($t - $prev)
    $prev = $t
    $vis = [P3]::VisibleCount([uint32]$p.Id)
    $bmp = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bmp.Save("$PWD\final_$t.png", [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
    Write-Host "t=${t}s visible=$vis captured final_$t.png"
}
Stop-Process -Id $p.Id -Force
Write-Host 'killed'
