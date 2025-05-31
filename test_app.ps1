#!/usr/bin/env pwsh
# Test script for Geek Commander

Write-Host "Testing Geek Commander Application" -ForegroundColor Cyan
Write-Host "===========================================" -ForegroundColor Cyan
Write-Host ""

# Check if executable exists
if (Test-Path "target/release/geekcommander.exe") {
    Write-Host "✅ Executable found: target/release/geekcommander.exe" -ForegroundColor Green
    
    # Get file info
    $file = Get-Item "target/release/geekcommander.exe"
    Write-Host "📁 Size: $([math]::Round($file.Length / 1KB, 2)) KB" -ForegroundColor Yellow
    Write-Host "📅 Modified: $($file.LastWriteTime)" -ForegroundColor Yellow
    Write-Host ""
    
    Write-Host "🔥 BUILD SUCCESSFUL!" -ForegroundColor Green
    Write-Host ""
    Write-Host "To run the application:" -ForegroundColor Cyan
    Write-Host "  .\target\release\geekcommander.exe" -ForegroundColor White
    Write-Host ""
    Write-Host "Available options:" -ForegroundColor Cyan
    Write-Host "  --help              Show help information" -ForegroundColor White
    Write-Host "  --config [file]     Use custom config file" -ForegroundColor White
    Write-Host "  --left-path [path]  Start left pane at path" -ForegroundColor White
    Write-Host "  --right-path [path] Start right pane at path" -ForegroundColor White
    Write-Host "  --monochrome        Use monochrome mode" -ForegroundColor White
    Write-Host ""
    Write-Host "📖 Documentation:" -ForegroundColor Cyan
    Write-Host "  - README.md - Complete user manual" -ForegroundColor White
    Write-Host "  - QUICKSTART.md - Quick setup guide" -ForegroundColor White
    Write-Host "  - CHANGELOG.md - Feature list and version history" -ForegroundColor White
    
} else {
    Write-Host "❌ Executable not found!" -ForegroundColor Red
    Write-Host "Run 'cargo build --release' to build the application" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🎉 Geek Commander - Norton Commander clone in Rust!" -ForegroundColor Magenta 