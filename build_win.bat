@echo off
REM Local Windows build helper for RadarVU (uncommitted, modeled on HybridEQ/build_win.bat).
REM Loads the MSVC x64 environment, then configures (if needed) and builds.
REM
REM   build_win.bat                 -> configure + build all targets
REM   build_win.bat --configure     -> configure only

setlocal

call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 (
    echo [ERROR] Failed to initialise the MSVC environment.
    exit /b 1
)

REM Rust toolchain for the DSP static library.
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

if not exist "build\CMakeCache.txt" (
    echo [RadarVU] Configuring...
    cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release -DRADARVU_BUILD_QA=ON
    if errorlevel 1 exit /b 1
)

if /i "%~1"=="--configure" exit /b 0

echo [RadarVU] Building all targets...
cmake --build build --config Release

exit /b %errorlevel%
