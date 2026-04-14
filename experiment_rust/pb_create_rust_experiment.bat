@echo off
setlocal enabledelayedexpansion

set DIR=%~dp0
set ROOT=%DIR%..
set TEMPDIR=%ROOT%\temp
set EXPDIR=%ROOT%\experiment_rust\rust_protobuf_cargo
set RUSTPROTODIR=%EXPDIR%\proto
set BUILD_FAILED=0

echo "%ROOT%\"

where cargo >nul 2>nul
if errorlevel 1 (
  echo cargo not found, please install Rust toolchain first.
  set BUILD_FAILED=1
  goto cleanup
)

echo "create temp proto files"
cd /d %ROOT%
java -jar %ROOT%\PbSplit.jar PbSplit
if errorlevel 1 (
  set BUILD_FAILED=1
  goto cleanup
)

if exist %RUSTPROTODIR% rmdir /s /q %RUSTPROTODIR%
mkdir %RUSTPROTODIR%

echo "copy proto files for rust experiment"
xcopy /y /q %TEMPDIR%\*.proto %RUSTPROTODIR%\ >nul
if errorlevel 1 (
  echo failed to copy proto files into rust experiment
  set BUILD_FAILED=1
  goto cleanup
)

echo "build rust protobuf experiment crate"
cd /d %EXPDIR%
call cargo build --release
if errorlevel 1 (
  set BUILD_FAILED=1
  goto cleanup
)

echo "package rust experiment crate"
call cargo package --allow-dirty --no-verify
if errorlevel 1 (
  set BUILD_FAILED=1
  goto cleanup
)

:cleanup
echo "remove temp proto files"
cd /d %ROOT%
if exist %TEMPDIR% rmdir /s /q %TEMPDIR%

if %BUILD_FAILED%==1 (
  echo "finished with errors"
  endlocal
  exit /b 1
)

echo "finished"
endlocal
