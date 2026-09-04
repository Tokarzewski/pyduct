# Uninstall the WentaZwcad auto-load registration.
# Run elevated: powershell -ExecutionPolicy Bypass -File uninstall.ps1

$ErrorActionPreference = 'Stop'

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'must run elevated'
}

Remove-Item 'HKLM:\SOFTWARE\ZWSOFT\ZWCAD\2021\en-US\Applications\WentaZwcad' -Recurse -ErrorAction SilentlyContinue
Remove-Item 'C:\ProgramData\WentaZwcad' -Recurse -ErrorAction SilentlyContinue
Write-Output 'WentaZwcad uninstalled (ribbon may remain in the ZWCAD profile until MENULOAD/CUI removes it).'
