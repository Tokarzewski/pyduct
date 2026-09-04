Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$root = [System.Windows.Automation.AutomationElement]::RootElement
# find ZWCAD windows by process
$proc = Get-Process ZWCAD -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $proc) { Write-Output 'no ZWCAD process'; exit 1 }

$win = [System.Windows.Automation.AutomationElement]::FromHandle($proc.MainWindowHandle)
Write-Output ("window: " + $win.Current.Name)

$walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
$found = New-Object System.Collections.ArrayList

function Walk($el, $depth) {
    if ($depth -gt 18) { return }
    try {
        $name = $el.Current.Name
        if ($name -match 'Wenta') { $script:found.Add($name + "  [" + $el.Current.ControlType.ProgrammaticName + "]") | Out-Null }
    } catch { return }
    $child = $walker.GetFirstChild($el)
    while ($child -ne $null) {
        Walk $child ($depth + 1)
        $child = $walker.GetNextSibling($child)
    }
}

Walk $win 0
Write-Output ("matches: " + $found.Count)
$found | ForEach-Object { Write-Output ("  " + $_) }
