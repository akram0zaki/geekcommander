# Geek Commander Build Script for Windows
# This script helps build and install the application

param(
    [switch]$Release,
    [switch]$Install,
    [switch]$Clean,
    [switch]$Test,
    [switch]$Help
)

function Show-Help {
    Write-Host "Geek Commander Build Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [OPTIONS]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -Release    Build optimized release version"
    Write-Host "  -Install    Install binary to system PATH"
    Write-Host "  -Clean      Clean build artifacts"
    Write-Host "  -Test       Run tests"
    Write-Host "  -Help       Show this help message"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\build.ps1              # Debug build"
    Write-Host "  .\build.ps1 -Release     # Release build"
    Write-Host "  .\build.ps1 -Test        # Run tests"
    Write-Host "  .\build.ps1 -Release -Install  # Build and install"
}

function Test-Cargo {
    try {
        $null = Get-Command cargo -ErrorAction Stop
        return $true
    }
    catch {
        return $false
    }
}

function Install-Rust {
    Write-Host "Rust not found. Installing Rust..." -ForegroundColor Yellow
    
    if (Test-Path "rustup-init.exe") {
        Write-Host "Found rustup-init.exe, running installer..." -ForegroundColor Green
        Start-Process -FilePath ".\rustup-init.exe" -ArgumentList "-y" -Wait
    } else {
        Write-Host "Downloading Rust installer..." -ForegroundColor Yellow
        try {
            Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
            Write-Host "Running Rust installer..." -ForegroundColor Green
            Start-Process -FilePath ".\rustup-init.exe" -ArgumentList "-y" -Wait
            Remove-Item "rustup-init.exe" -ErrorAction SilentlyContinue
        }
        catch {
            Write-Host "Failed to download Rust installer: $($_.Exception.Message)" -ForegroundColor Red
            Write-Host "Please install Rust manually from https://rustup.rs/" -ForegroundColor Yellow
            exit 1
        }
    }
    
    # Refresh PATH
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
    
    Write-Host "Please restart your terminal and run this script again." -ForegroundColor Yellow
    exit 0
}

function Build-Project {
    param([bool]$IsRelease)
    
    Write-Host "Building Geek Commander..." -ForegroundColor Cyan
    
    if ($IsRelease) {
        Write-Host "Building release version..." -ForegroundColor Green
        cargo build --release
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Build successful!" -ForegroundColor Green
            Write-Host "Binary location: .\target\release\geekcommander.exe" -ForegroundColor Yellow
        }
    } else {
        Write-Host "Building debug version..." -ForegroundColor Green
        cargo build
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Build successful!" -ForegroundColor Green
            Write-Host "Binary location: .\target\debug\geekcommander.exe" -ForegroundColor Yellow
        }
    }
    
    return $LASTEXITCODE -eq 0
}

function Install-Binary {
    param([bool]$IsRelease)
    
    $binaryPath = if ($IsRelease) { ".\target\release\geekcommander.exe" } else { ".\target\debug\geekcommander.exe" }
    
    if (-not (Test-Path $binaryPath)) {
        Write-Host "Binary not found at $binaryPath. Please build first." -ForegroundColor Red
        return $false
    }
    
    Write-Host "Installing binary to system..." -ForegroundColor Cyan
    
    # Try to install to user's local bin directory first
    $localBin = "$env:USERPROFILE\.cargo\bin"
    if (Test-Path $localBin) {
        try {
            Copy-Item $binaryPath "$localBin\geekcommander.exe" -Force
            Write-Host "Installed to $localBin\geekcommander.exe" -ForegroundColor Green
            Write-Host "You can now run 'geekcommander' from anywhere" -ForegroundColor Yellow
            return $true
        }
        catch {
            Write-Host "Failed to install to user directory: $($_.Exception.Message)" -ForegroundColor Red
        }
    }
    
    # Fallback: try system directory (requires admin)
    try {
        Copy-Item $binaryPath "C:\Windows\System32\geekcommander.exe" -Force
        Write-Host "Installed to C:\Windows\System32\geekcommander.exe" -ForegroundColor Green
        Write-Host "You can now run 'geekcommander' from anywhere" -ForegroundColor Yellow
        return $true
    }
    catch {
        Write-Host "Failed to install to system directory (requires admin privileges)" -ForegroundColor Red
        Write-Host "Manual installation:" -ForegroundColor Yellow
        Write-Host "  1. Copy $binaryPath to a directory in your PATH" -ForegroundColor White
        Write-Host "  2. Or add the target directory to your PATH" -ForegroundColor White
        return $false
    }
}

function Clean-Build {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Cyan
    cargo clean
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Clean completed!" -ForegroundColor Green
    }
}

function Run-Tests {
    Write-Host "Running tests..." -ForegroundColor Cyan
    cargo test
    if ($LASTEXITCODE -eq 0) {
        Write-Host "All tests passed!" -ForegroundColor Green
    } else {
        Write-Host "Some tests failed!" -ForegroundColor Red
    }
}

# Main script logic
if ($Help) {
    Show-Help
    exit 0
}

# Check if Rust is installed
if (-not (Test-Cargo)) {
    Install-Rust
}

# Perform requested actions
if ($Clean) {
    Clean-Build
}

if ($Test) {
    Run-Tests
}

# Default action: build
if (-not $Clean -and -not $Test -and -not $Help) {
    $buildSuccess = Build-Project -IsRelease $Release
    
    if ($buildSuccess -and $Install) {
        Install-Binary -IsRelease $Release
    }
    
    if ($buildSuccess) {
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Cyan
        if ($Release) {
            Write-Host "  Run: .\target\release\geekcommander.exe" -ForegroundColor White
        } else {
            Write-Host "  Run: .\target\debug\geekcommander.exe" -ForegroundColor White
            Write-Host "  Or:  cargo run" -ForegroundColor White
        }
        
        if (-not $Install) {
            Write-Host "  Install: .\build.ps1 -Release -Install" -ForegroundColor White
        }
    }
} 