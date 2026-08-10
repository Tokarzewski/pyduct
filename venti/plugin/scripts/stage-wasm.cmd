@echo off
REM Stage the not-yet-built venti WASM core beside the plugin DLL (issue #17).
REM Run from the venti crate root (build-wasm first), or point at an artifact:
REM   plugin\scripts\stage-wasm.cmd <path-to-venti.wasm>
setlocal
set SRC=%1
if "%SRC%"=="" set SRC=target\wasm32-wasip1\release\venti.wasm
if not exist "%SRC%" (
  echo [venti] venti.wasm not found at %SRC% - run ..\scripts\build-wasm.sh --release first.
  exit /b 1
)
copy /Y "%SRC%" plugin\bin\Release\venti.wasm >nul
echo [venti] staged venti.wasm into plugin\bin\Release
exit /b 0
