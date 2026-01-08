# Development runner: Build DLL and run Rust example
# Usage: .\dev.ps1 [-Release] [-NoExample] [-NoRebuild]

param(
    [switch]$Release,
    [switch]$NoExample,
    [switch]$NoRebuild
)

$ErrorActionPreference = "Stop"

# ---------------------------------------------
# Optional proxy config (NOT committed to git)
# If the file exists, it will be dot-sourced.
# ---------------------------------------------
$ProxyConfigPath = Join-Path $PSScriptRoot "dev.proxy.ps1"
if (Test-Path $ProxyConfigPath) {
    try {
        . $ProxyConfigPath
        # NOTE: proxy settings are expected to be defined inside dev.proxy.ps1
    }
    catch {
        Write-Host "Failed to load proxy config: $ProxyConfigPath" -ForegroundColor Red
        throw
    }
}

# Colors for output
$Green = @{ ForegroundColor = "Green" }
$Yellow = @{ ForegroundColor = "Yellow" }
$Red = @{ ForegroundColor = "Red" }

function Write-Title {
    param([string]$Message)
    Write-Host "`n" -NoNewline
    Write-Host "========================================" @Yellow
    Write-Host $Message @Yellow
    Write-Host "========================================`n" @Yellow
}

function Write-Error-Exit {
    param([string]$Message)
    Write-Host $Message @Red
    exit 1
}

# Determine preset and example mode
$Preset = if ($Release) { "msvc-release" } else { "msvc-debug" }
$ExampleBuild = if ($NoExample) { $false } else { $true }

Write-Title "spoutdx-ffi Development Build"
Write-Host "Preset: $Preset" @Green
Write-Host "Build Example: $ExampleBuild" @Green

# Step 1: Build C++ DLL
if (-not $NoRebuild) {
    Write-Title "Step 1: Building C++ DLL"
    
    try {
        Push-Location $PSScriptRoot
        
        # Configure
        Write-Host "Configuring CMake..." @Green
        cmake --preset $Preset 2>&1
        if ($LASTEXITCODE -ne 0) { Write-Error-Exit "CMake configuration failed" }
        
        # Build
        Write-Host "Building DLL..." @Green
        cmake --build --preset $Preset 2>&1
        if ($LASTEXITCODE -ne 0) { Write-Error-Exit "CMake build failed" }
        
        Write-Host "[OK] C++ DLL built successfully" @Green
    }
    finally {
        Pop-Location
    }
} else {
    Write-Host "[NG] Skipping DLL rebuild" @Yellow
}

# Step 2: Run Rust Example
if ($ExampleBuild) {
    Write-Title "Step 2: Running Rust Example"
    
    try {
        Push-Location (Join-Path $PSScriptRoot "examples")
        
        Write-Host "Running example..." @Green
        # Suppress warnings from cargo build scripts
        $WarningPreference = "SilentlyContinue"
        cargo run $(if ($Release) { "--release" } else { })
        $ExitCode = $LASTEXITCODE
        $WarningPreference = "Continue"
        
        if ($ExitCode -ne 0) {
            Write-Error-Exit "Example execution failed"
        }
        
        Write-Host "`n [OK] Example ran successfully" @Green
    }
    finally {
        Pop-Location
    }
} else {
    Write-Host "[NG] Skipping example execution" @Yellow
}

Write-Title "Build Complete!"
Write-Host "DLL Location: .\out\build\$Preset\$(if ($Release) { 'Release' } else { 'Debug' })\spoutdx_ffi.dll" @Green
Write-Host "Example Binary: .\examples\target\$(if ($Release) { 'release' } else { 'debug' })\ping.exe" @Green
