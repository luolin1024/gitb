# gitb one-line installer for Windows
# Usage (PowerShell):
#   irm https://github.com/luolin1024/git-batch/raw/main/install.ps1 | iex

$ErrorActionPreference = "Stop"
$VERSION = "v0.3.0"
$REPO = "luolin1024/git-batch"
$FILE = "gitb-x86_64-windows.exe"

# Install to ~/bin (create if needed)
$InstallDir = "$HOME\bin"
if (!(Test-Path $InstallDir)) { New-Item -ItemType Directory -Force $InstallDir | Out-Null }

$Dest = "$InstallDir\gitb.exe"
$Url = "https://github.com/$REPO/releases/download/$VERSION/$FILE"

Write-Host "⬇️  Downloading gitb $VERSION..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $Url -OutFile $Dest

# Add ~/bin to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "✅ Added $InstallDir to PATH (restart terminal to take effect)." -ForegroundColor Green
}

Write-Host "✅ Installed to: $Dest" -ForegroundColor Green
Write-Host "🚀 Run 'gitb --version' to verify." -ForegroundColor Cyan
