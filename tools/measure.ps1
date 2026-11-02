Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Bitmap]::FromFile('D:\tools\ctf\script\DS_Desktop\aday\razer-driver\rz-app\assets\device_dav3pro.png')
$ncols = 50
$nrows = 50
$cw = [double]$img.Width / $ncols
$ch = [double]$img.Height / $nrows
Write-Output ("size {0}x{1} cell {2:N1}x{3:N1}" -f $img.Width, $img.Height, $cw, $ch)
$out = @()
for ($r = 0; $r -lt $nrows; $r++) {
    $line = ''
    for ($c = 0; $c -lt $ncols; $c++) {
        $px = $img.GetPixel([int](($c + 0.5) * $cw), [int](($r + 0.5) * $ch))
        $lum = 0.299 * $px.R + 0.587 * $px.G + 0.114 * $px.B
        if ($lum -lt 40) { $line += '#' }
        elseif ($lum -lt 80) { $line += 'O' }
        elseif ($lum -lt 140) { $line += 'o' }
        elseif ($lum -lt 210) { $line += '.' }
        else { $line += ' ' }
    }
    $out += ('{0,2} {1}' -f $r, $line)
}
$out -join "`n"
Write-Output '--- extents (y%, x0%, x1%) ---'
for ($r = 0; $r -lt $nrows; $r += 2) {
    $x0 = -1
    $x1 = -1
    for ($c = 0; $c -lt $ncols; $c++) {
        $px = $img.GetPixel([int](($c + 0.5) * $cw), [int](($r + 0.5) * $ch))
        $lum = 0.299 * $px.R + 0.587 * $px.G + 0.114 * $px.B
        if ($lum -lt 150) {
            if ($x0 -lt 0) { $x0 = $c }
            $x1 = $c
        }
    }
    if ($x0 -ge 0) { Write-Output ('{0,3}%  {1,4}%  {2,4}%' -f [int](100 * $r / $nrows), [int](100 * $x0 / $ncols), [int](100 * ($x1 + 1) / $ncols)) }
}
$img.Dispose()
