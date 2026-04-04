# Headway CLI installer for Windows
# Usage: irm https://raw.githubusercontent.com/Code-Barru/headway/main/scripts/install.ps1 | iex
#
# Options (via parameters or environment variables):
#   -Version      / $env:HEADWAY_VERSION       Specific version (e.g. "0.3.0")
#   -InstallDir   / $env:HEADWAY_INSTALL_DIR   Custom install directory
#   -Yes          / $env:HEADWAY_YES            Non-interactive mode

[CmdletBinding()]
param(
    [string]$Version = "",
    [string]$InstallDir = "",
    [switch]$Yes,
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

$Repo = "Code-Barru/headway"
$BinaryName = "headway"
$GitHubApi = "https://api.github.com/repos/$Repo/releases"
$GitHubDownload = "https://github.com/$Repo/releases/download"

# --- Environment variable overrides (for irm | iex compatibility) ---
if ($env:HEADWAY_VERSION -and -not $Version) { $Version = $env:HEADWAY_VERSION }
if ($env:HEADWAY_INSTALL_DIR -and -not $InstallDir) { $InstallDir = $env:HEADWAY_INSTALL_DIR }
if ($env:HEADWAY_YES -eq "true") { $Yes = [switch]::Present }

if (-not $InstallDir) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "headway\bin"
}

# --- Helpers ---
function Write-Info { param([string]$Message) Write-Host "info " -ForegroundColor Blue -NoNewline; Write-Host $Message }
function Write-Warn { param([string]$Message) Write-Host "warn " -ForegroundColor Yellow -NoNewline; Write-Host $Message }
function Write-Err  { param([string]$Message) Write-Host "error " -ForegroundColor Red -NoNewline; Write-Host $Message }
function Write-Ok   { param([string]$Message) Write-Host "done " -ForegroundColor Green -NoNewline; Write-Host $Message }

function Show-Usage {
    Write-Host @"
Headway CLI installer for Windows

Usage:
    .\install.ps1 [OPTIONS]

Options:
    -Version VER      Install a specific version (e.g. 0.3.0)
    -InstallDir DIR   Custom install directory (default: %LOCALAPPDATA%\headway\bin)
    -Yes              Non-interactive mode (accept all defaults)
    -Help             Show this help message

Environment variables:
    HEADWAY_VERSION       Same as -Version
    HEADWAY_INSTALL_DIR   Same as -InstallDir
    HEADWAY_YES           Set to "true" for non-interactive mode

Examples:
    irm .../install.ps1 | iex
    `$env:HEADWAY_VERSION="0.3.0"; irm .../install.ps1 | iex
    .\install.ps1 -Version 0.3.0
"@
    exit 0
}

if ($Help) { Show-Usage }

# --- Architecture detection ---
function Get-TargetArch {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default { throw "Unsupported architecture: $arch" }
    }
}

# --- Version resolution ---
function Resolve-LatestVersion {
    Write-Info "Fetching latest version..."
    try {
        $releases = Invoke-RestMethod -Uri $GitHubApi -Headers @{ 'User-Agent' = 'headway-installer' } -UseBasicParsing
        $cliRelease = $releases | Where-Object { $_.tag_name -like 'cli@*' } | Select-Object -First 1
        if (-not $cliRelease) { throw "No CLI release found" }
        return $cliRelease.tag_name
    }
    catch {
        throw "Failed to fetch latest CLI release. Try passing -Version explicitly. ($_)"
    }
}

# --- Checksum verification ---
function Test-Checksum {
    param(
        [string]$ArchivePath,
        [string]$ArchiveName,
        [string]$Tag
    )

    $checksumUrl = "$GitHubDownload/$Tag/checksums.sha256"
    $checksumFile = Join-Path $tmpDir "checksums.sha256"

    try {
        Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumFile -UseBasicParsing 2>$null
    }
    catch {
        Write-Warn "Checksums file not available for this release, skipping verification."
        return
    }

    $checksumContent = Get-Content $checksumFile
    $expectedLine = $checksumContent | Where-Object { $_ -match $ArchiveName }

    if (-not $expectedLine) {
        Write-Warn "Archive not found in checksums file, skipping verification."
        return
    }

    $expected = ($expectedLine -split '\s+')[0]
    $actual = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash.ToLower()

    if ($actual -ne $expected) {
        throw "Checksum verification failed! Expected: $expected, Got: $actual"
    }

    Write-Ok "Checksum verified."
}

# --- PATH configuration ---
function Add-ToPath {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')

    if ($userPath -and $userPath.Split(';') -contains $InstallDir) {
        return
    }

    if (-not $Yes) {
        $response = Read-Host "Add $InstallDir to PATH? [Y/n]"
        if ($response -match '^[nN]') {
            Write-Warn "Skipping PATH configuration. Add it manually to your PATH."
            return
        }
    }

    if ($userPath) {
        $newPath = "$InstallDir;$userPath"
    }
    else {
        $newPath = $InstallDir
    }

    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    $env:Path = "$InstallDir;$env:Path"
    Write-Ok "Added $InstallDir to user PATH."
    Write-Info "Restart your terminal for the change to take effect."
}

# --- Main ---
function Install-Headway {
    Write-Host "`nHeadway CLI Installer`n" -ForegroundColor White

    # Detect architecture
    $targetArch = Get-TargetArch
    $target = "$targetArch-pc-windows-msvc"

    Write-Info "Platform: Windows ($targetArch)"
    Write-Info "Target: $target"

    # Resolve version
    if ($Version) {
        $tag = "cli@v$Version"
        Write-Info "Using specified version: $Version"
    }
    else {
        $tag = Resolve-LatestVersion
        $Version = $tag -replace '^cli@v', ''
        Write-Info "Latest version: $Version"
    }

    $archiveName = "$BinaryName-$target.zip"
    $downloadUrl = "$GitHubDownload/$tag/$archiveName"

    # Create temp directory
    $script:tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "headway-install-$([System.Guid]::NewGuid().ToString('N').Substring(0,8))"
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        # Download
        $archivePath = Join-Path $tmpDir $archiveName
        Write-Info "Downloading $archiveName..."
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
        }
        catch {
            throw "Download failed. Check that version $Version exists for target $target. ($_)"
        }

        # Verify checksum
        Test-Checksum -ArchivePath $archivePath -ArchiveName $archiveName -Tag $tag

        # Extract
        Write-Info "Extracting..."
        Expand-Archive -Path $archivePath -DestinationPath $tmpDir -Force

        # Install
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        $sourceBinary = Join-Path $tmpDir "$BinaryName-$target" "$BinaryName.exe"
        $destBinary = Join-Path $InstallDir "$BinaryName.exe"
        Copy-Item -Path $sourceBinary -Destination $destBinary -Force

        # Verify
        try {
            $installedVersion = & $destBinary --version 2>&1
            Write-Ok "Installed $installedVersion to $destBinary"
        }
        catch {
            Write-Ok "Installed headway v$Version to $destBinary"
        }

        # Configure PATH
        Add-ToPath

        Write-Host "`nRun 'headway --help' to get started.`n"
    }
    finally {
        # Cleanup
        if (Test-Path $tmpDir) {
            Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

Install-Headway
