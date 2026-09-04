# Dump public API of ZwRibbonControls.dll (WPF ribbon of ZWCAD 2021)
$ErrorActionPreference = 'Stop'
$zw = 'C:\Program Files\ZWSOFT\ZWCAD 2021'
Set-Location $zw
[Environment]::CurrentDirectory = $zw

$a = [Reflection.Assembly]::LoadFrom((Join-Path $zw 'ZwRibbonControls.dll'))

Write-Output '=== exported types (Ribbon-related) ==='
$types = $a.GetExportedTypes() | Where-Object { $_.FullName -match 'Ribbon' }
$types | ForEach-Object { $_.FullName } | Sort-Object

foreach ($name in @('RibbonControl','RibbonTab','RibbonGroup','RibbonButton','RibbonCommandButton')) {
    $t = $a.GetExportedTypes() | Where-Object { $_.Name -eq $name }
    if (-not $t) { Write-Output ("(no exported type named " + $name + ")"); continue }
    $t = $t[0]
    Write-Output ''
    Write-Output ('===== ' + $t.FullName + ' =====')
    Write-Output '--- ctors ---'
    $t.GetConstructors() | ForEach-Object { $_.ToString() }
    Write-Output '--- public props ---'
    $t.GetProperties() | Where-Object { $_.DeclaringType -eq $t } | ForEach-Object { $_.ToString() }
    Write-Output '--- public methods ---'
    $t.GetMethods() | Where-Object { $_.DeclaringType -eq $t -and -not $_.IsSpecialName } | ForEach-Object { $_.ToString() }
}
