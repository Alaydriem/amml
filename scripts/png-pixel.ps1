# Prints the colour of one pixel of a PNG file as "r g b". Read-only. Exit 1 when the file cannot be read or the
# point is outside the image.
param(
  [Parameter(Mandatory = $true)][string]$Path,
  [Parameter(Mandatory = $true)][int]$X,
  [Parameter(Mandatory = $true)][int]$Y
)
Add-Type -AssemblyName System.Drawing
try { $image = New-Object System.Drawing.Bitmap $Path } catch { "cannot read $Path"; exit 1 }
try {
  if ($X -ge $image.Width -or $Y -ge $image.Height) { "($X,$Y) is outside the $($image.Width)x$($image.Height) image"; exit 1 }
  $pixel = $image.GetPixel($X, $Y)
  "{0} {1} {2}" -f $pixel.R, $pixel.G, $pixel.B
} finally {
  $image.Dispose()
}
