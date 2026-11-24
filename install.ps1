# LLVM IR Studio (llvm-ir-ls) Windows Installer
# Installs llvm-ir-ls to %LOCALAPPDATA%\Programs\llvm-ir-ls and adds to user PATH.
param (
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$Repo = "SohamTilekar/llvm-ir-studio"
$BinaryName = "llvm-ir-ls.exe"
$InstallDir = "$env:LOCALAPPDATA\Programs\llvm-ir-ls"

Write-Host "==> LLVM IR Studio — Windows Language Server Installer" -ForegroundColor Cyan
Write-Host "--------------------------------------------------------"

# 1. Determine Version
if (-not $Version) {
    try {
        $ReleaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{"Accept"="application/vnd.github+json"}
        $Version = $ReleaseInfo.tag_name
    } catch {
        $Version = "v0.1.0"
    }
}

if (-not $Version.StartsWith("v")) {
    $Tag = "v$Version"
} else {
    $Tag = $Version
}

$Target = "x86_64-pc-windows-msvc"
$ArchiveName = "llvm-ir-ls-$Tag-$Target.zip"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/$ArchiveName"

Write-Host "==> Downloading $ArchiveName..." -ForegroundColor Green
$TempZip = Join-Path $env:TEMP $ArchiveName
$TempExtract = Join-Path $env:TEMP "llvm-ir-ls-extract"

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip
    if (Test-Path $TempExtract) { Remove-Item -Recurse -Force $TempExtract }
    Expand-Archive -Path $TempZip -DestinationPath $TempExtract -Force

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    Copy-Item (Join-Path $TempExtract $BinaryName) -Destination (Join-Path $InstallDir $BinaryName) -Force
    Write-Host "==> Installed llvm-ir-ls.exe to $InstallDir" -ForegroundColor Green

    # Add to User PATH if not present
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$UserPath", "User")
        Write-Host "==> Added $InstallDir to User PATH." -ForegroundColor Yellow
        Write-Host "    (Please restart your terminal or editor for changes to take effect)" -ForegroundColor Yellow
    }

    Write-Host "==> Verification:" -ForegroundColor Cyan
    & (Join-Path $InstallDir $BinaryName) --version
} catch {
    Write-Warning "Direct release download failed: $_"
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "==> Cargo detected. Building and installing from crates.io/git..." -ForegroundColor Green
        cargo install --git "https://github.com/$Repo.git" llvm-ir-ls
    } else {
        Write-Error "Failed to install llvm-ir-ls. Please install manually from https://github.com/$Repo/releases"
    }
} finally {
    if (Test-Path $TempZip) { Remove-Item -Force $TempZip }
    if (Test-Path $TempExtract) { Remove-Item -Recurse -Force $TempExtract }
}

Write-Host "--------------------------------------------------------"
Write-Host "Setup complete!" -ForegroundColor Cyan
