@echo off
setlocal enabledelayedexpansion

:: build.bat - build the Rust FFI workspace, then the TheDomain UE 5.8 project.
::
:: Usage:
::   build.bat                 Build the Development Editor target (default)
::   build.bat Shipping        Build a Shipping configuration instead
::
:: If Unreal Engine 5.8 isn't auto-detected, set UE_ROOT first, e.g.:
::   set UE_ROOT=C:\Program Files\Epic Games\UE_5.8

set "REPO_ROOT=%~dp0"
set "CONFIG=%~1"
if "%CONFIG%"=="" set "CONFIG=Development"

echo === The Domain build ===
echo Configuration: %CONFIG%

:: --- 1. Build the Rust FFI workspace -------------------------------------
:: The Cargo workspace root is the repo root itself (see Cargo.toml), not
:: core/ — cargo build output lands in <repo-root>/target/release, which is
:: what NexusCore.Build.cs links against.
echo.
echo [1/3] Building Rust workspace (release)...
pushd "%REPO_ROOT%"
cargo build --release --workspace
if errorlevel 1 (
    echo.
    echo Rust build failed.
    popd
    exit /b 1
)
popd

:: --- 2. Locate the Unreal Engine 5.8 install -----------------------------
echo.
echo [2/3] Locating Unreal Engine 5.8...
if not defined UE_ROOT (
    for /f "usebackq tokens=2,*" %%A in (`reg query "HKLM\SOFTWARE\EpicGames\Unreal Engine\5.8" /v InstalledDirectory 2^>nul`) do set "UE_ROOT=%%B"
)
if not defined UE_ROOT (
    for /f "usebackq tokens=2,*" %%A in (`reg query "HKCU\SOFTWARE\EpicGames\Unreal Engine\5.8" /v InstalledDirectory 2^>nul`) do set "UE_ROOT=%%B"
)
if not defined UE_ROOT (
    echo.
    echo Could not auto-detect Unreal Engine 5.8.
    echo Set UE_ROOT and re-run, e.g.:
    echo   set UE_ROOT=C:\Program Files\Epic Games\UE_5.8
    exit /b 1
)

:: Strip any quotes the user (or registry value) may have included, and a
:: trailing backslash, so the quoting added below doesn't get doubled up.
set "UE_ROOT=%UE_ROOT:"=%"
if "%UE_ROOT:~-1%"=="\" set "UE_ROOT=%UE_ROOT:~0,-1%"
echo Found engine at: %UE_ROOT%

set "UPROJECT=%REPO_ROOT%engine\TheDomain.uproject"
set "BUILD_BAT=%UE_ROOT%\Engine\Build\BatchFiles\Build.bat"
set "GEN_BAT=%UE_ROOT%\Engine\Build\BatchFiles\GenerateProjectFiles.bat"

if not exist "%BUILD_BAT%" (
    echo.
    echo Build.bat not found at "%BUILD_BAT%" - is UE_ROOT correct?
    exit /b 1
)

:: --- 3. Generate project files (optional) and build ----------------------
:: GenerateProjectFiles.bat only produces the .sln/.vcxproj for opening the
:: project in Visual Studio — Build.bat/UBT compiles straight from
:: Target.cs/Build.cs and doesn't need it. Some engine installs don't ship
:: it at this path, so skip it rather than failing the whole build.
echo.
echo [3/3] Generating project files and building TheDomainEditor (%CONFIG%)...
if exist "%GEN_BAT%" (
    call "%GEN_BAT%" -project="%UPROJECT%" -game
    if errorlevel 1 (
        echo.
        echo Generate project files failed ^(continuing — not required to build^).
    )
) else (
    echo GenerateProjectFiles.bat not found at "%GEN_BAT%" — skipping ^(not required to build^).
    echo To open the project in Visual Studio, right-click TheDomain.uproject and
    echo choose "Generate Visual Studio project files" instead.
)

call "%BUILD_BAT%" TheDomainEditor Win64 %CONFIG% -Project="%UPROJECT%" -WaitMutex
if errorlevel 1 (
    echo.
    echo UE5 build failed.
    exit /b 1
)

echo.
echo === Build succeeded ===
endlocal
