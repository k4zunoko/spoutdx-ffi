# Distribution builder: Build Release and package DLL, LIB, and headers
# Usage: .\dist.ps1 [-NoRebuild]

param(
    [switch]$NoRebuild
)

$ErrorActionPreference = "Stop"

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

# Determine paths
$RootDir = $PSScriptRoot
$Preset = "msvc-release"
$BuildDir = Join-Path $RootDir "out\build\$Preset\Release"
$DistDir = Join-Path $RootDir "dist\spoutdx_ffi"

Write-Title "spoutdx-ffi Distribution Builder (Release)"
Write-Host "Root Directory: $RootDir" @Green
Write-Host "Build Directory: $BuildDir" @Green
Write-Host "Distribution Directory: $DistDir" @Green

# Step 1: Build C++ DLL (Release)
if (-not $NoRebuild) {
    Write-Title "Step 1: Building C++ DLL (Release)"
    
    try {
        Push-Location $RootDir
        
        # Configure
        Write-Host "Configuring CMake..." @Green
        cmake --preset $Preset 2>&1
        if ($LASTEXITCODE -ne 0) { Write-Error-Exit "CMake configuration failed" }
        
        # Build
        Write-Host "Building DLL (Release)..." @Green
        cmake --build --preset $Preset --config Release 2>&1
        if ($LASTEXITCODE -ne 0) { Write-Error-Exit "CMake build failed" }
        
        Write-Host "[OK] C++ DLL built successfully" @Green
    }
    finally {
        Pop-Location
    }
} else {
    Write-Host "[NG] Skipping DLL rebuild" @Yellow
}

# Step 2: Create distribution folder
Write-Title "Step 2: Creating Distribution Folder"

try {
    # Remove existing dist folder
    if (Test-Path $DistDir) {
        Write-Host "Removing existing distribution folder..." @Yellow
        Remove-Item -Path $DistDir -Recurse -Force
    }
    
    # Create directory structure
    Write-Host "Creating directory structure..." @Green
    New-Item -ItemType Directory -Path $DistDir | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $DistDir "bin") | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $DistDir "lib") | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $DistDir "include") | Out-Null
    
    Write-Host "[OK] Directory structure created" @Green
}
catch {
    Write-Error-Exit "Failed to create distribution directory: $_"
}

# Step 3: Copy files
Write-Title "Step 3: Copying Files"

try {
    # Check if build directory exists
    if (-not (Test-Path $BuildDir)) {
        Write-Error-Exit "Build directory not found: $BuildDir`nPlease ensure the build completed successfully"
    }
    
    # Copy DLL
    $DllFile = Join-Path $BuildDir "spoutdx_ffi.dll"
    if (Test-Path $DllFile) {
        Write-Host "Copying DLL..." @Green
        Copy-Item -Path $DllFile -Destination (Join-Path $DistDir "bin\")
        Write-Host "[OK] spoutdx_ffi.dll copied" @Green
    }
    else {
        Write-Error-Exit "DLL not found: $DllFile"
    }
    
    # Copy LIB file
    $LibFile = Join-Path $BuildDir "spoutdx_ffi.lib"
    if (Test-Path $LibFile) {
        Write-Host "Copying LIB..." @Green
        Copy-Item -Path $LibFile -Destination (Join-Path $DistDir "lib\")
        Write-Host "[OK] spoutdx_ffi.lib copied" @Green
    }
    else {
        Write-Host "Warning: LIB file not found: $LibFile" @Yellow
    }
    
    # Copy header files
    $IncludeDir = Join-Path $RootDir "include\spoutdx_ffi"
    if (Test-Path $IncludeDir) {
        Write-Host "Copying header files..." @Green
        Get-ChildItem -Path $IncludeDir -Filter "*.h" | ForEach-Object {
            Copy-Item -Path $_.FullName -Destination (Join-Path $DistDir "include\")
            Write-Host "  - $($_.Name)" @Green
        }
        Write-Host "[OK] Header files copied" @Green
    }
    else {
        Write-Error-Exit "Include directory not found: $IncludeDir"
    }
}
catch {
    Write-Error-Exit "Failed to copy files: $_"
}

# Step 4: Summary
Write-Title "Distribution Complete!"
Write-Host "Distribution folder: $DistDir" @Green
Write-Host "" @Green
Write-Host "Contents:" @Green
Write-Host "  bin/  - spoutdx_ffi.dll" @Green
Write-Host "  lib/  - spoutdx_ffi.lib" @Green
Write-Host "  include/ - Header files" @Green
Write-Host "" @Green

# Display directory tree
Write-Host "Directory structure:" @Green
if (Get-Command tree -ErrorAction SilentlyContinue) {
    tree /F $DistDir
}
else {
    Get-ChildItem -Path $DistDir -Recurse | ForEach-Object {
        $Indent = ("  " * (($_.FullName.Split('\').Count - $DistDir.Split('\').Count)))
        Write-Host "$Indent$($_.Name)" @Green
    }
}
