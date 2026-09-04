@echo off
rem ------------------------------------------------------------------
rem Build WentaZwcad.dll for ZWCAD 2021 (x64, .NET Framework 4.x).
rem Wenta.Core (the C# port of the wenta library) is compiled *into*
rem the plugin DLL — single-file deployment, no dependency resolution
rem issues with ZWCAD's LoadFrom context.
rem Prerequisite: csharp\build.cmd has run (it builds the standalone
rem core + parity tests).
rem ------------------------------------------------------------------
setlocal

set CSC="C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin\Roslyn\csc.exe"
set ZW=C:\Program Files\ZWSOFT\ZWCAD 2021
set FW=C:\Windows\Microsoft.NET\Framework64\v4.0.30319
set REPO=%~dp0..
set SRC=%~dp0WentaZwcad
set CORE=%REPO%\csharp\Wenta.Core
set OUT=%~dp0bin

if not exist "%OUT%" mkdir "%OUT%"

%CSC% ^
  -nologo -target:library -platform:x64 ^
  -optimize+ -deterministic ^
  -out:"%OUT%\WentaZwcad.dll" ^
  -r:"%FW%\System.dll" ^
  -r:"%FW%\System.Core.dll" ^
  -r:"%FW%\System.Windows.Forms.dll" ^
  -r:"%FW%\System.Drawing.dll" ^
  -r:"%FW%\System.Web.Extensions.dll" ^
  -r:"%ZW%\ZwManaged.dll" ^
  -r:"%ZW%\ZwDatabaseMgd.dll" ^
  "%SRC%\Commands.cs" ^
  "%SRC%\Plugin.cs" ^
  "%SRC%\WentaPanel.cs" ^
  "%CORE%\Units.cs" ^
  "%CORE%\Fluid.cs" ^
  "%CORE%\Geometry.cs" ^
  "%CORE%\Physics.cs" ^
  "%CORE%\StandardSizes.cs" ^
  "%CORE%\FittingsLibrary.cs" ^
  "%CORE%\Elbow.cs" ^
  "%CORE%\Components.cs" ^
  "%CORE%\Network.cs" ^
  "%CORE%\Solver.cs" ^
  "%CORE%\Sizing.cs" ^
  "%CORE%\Catalog.cs" ^
  "%CORE%\Bom.cs" ^
  "%CORE%\Balancing.cs" ^
  "%CORE%\Room.cs"

if errorlevel 1 ( echo BUILD FAILED & exit /b 1 )

copy /y "%REPO%\csharp\catalogs\example-generic.json" "%OUT%" >nul

echo BUILD OK: %OUT%\WentaZwcad.dll  (+ example-generic.json)
endlocal
