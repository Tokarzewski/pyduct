# Dump public API of ZwCui.dll (ZwSoft.ZwCAD.Ribbon namespace)
$ErrorActionPreference = 'Continue'
$zw = 'C:\Program Files\ZWSOFT\ZWCAD 2021'
Set-Location $zw
[Environment]::CurrentDirectory = $zw

$a = [Reflection.Assembly]::LoadFrom((Join-Path $zw 'ZwCui.dll'))
if (-not $a) { throw 'load failed' }

Write-Output '=== all exported types ==='
$a.GetExportedTypes() | ForEach-Object { $_.FullName } | Sort-Object | Out-String -Width 200

Write-Output '=== members of interesting types ==='
foreach ($t in $a.GetExportedTypes()) {
    if ($t.FullName -match 'Ribbon|Cui|MenuGroup') {
        Write-Output ('===== ' + $t.FullName + ' (base: ' + $t.BaseType.FullName + ') =====')
        Write-Output '--- ctors ---'
        $t.GetConstructors() | ForEach-Object { $_.ToString() }
        Write-Output '--- static props/methods ---'
        $t.GetProperties() | Where-Object { $_.GetAccessors($true)[0].IsStatic } | ForEach-Object { 'static ' + $_.ToString() }
        $t.GetMethods() | Where-Object { $_.IsStatic -and -not $_.IsSpecialName } | ForEach-Object { 'static ' + $_.ToString() }
        Write-Output '--- instance props ---'
        $t.GetProperties() | Where-Object { -not $_.GetAccessors($true)[0].IsStatic } | ForEach-Object { $_.ToString() }
        Write-Output '--- instance methods ---'
        $t.GetMethods() | Where-Object { -not $_.IsStatic -and -not $_.IsSpecialName } | ForEach-Object { $_.ToString() }
        Write-Output ''
    }
}
