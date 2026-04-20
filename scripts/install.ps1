# Gapline CLI installer for Windows
# Usage: irm https://raw.githubusercontent.com/Code-Barru/gapline/main/scripts/install.ps1 | iex
#
# Options (via environment variables):
#   $env:GAPLINE_VERSION       Specific version (e.g. "0.3.0")
#   $env:GAPLINE_INSTALL_DIR   Custom install directory
#   $env:GAPLINE_YES           Set to "true" for non-interactive mode

& {
    $ErrorActionPreference = 'Stop'

    $Repo = "Code-Barru/gapline"
    $BinaryName = "gapline"
    $GitHubApi = "https://api.github.com/repos/$Repo/releases"
    $GitHubDownload = "https://github.com/$Repo/releases/download"

    $Version = if ($env:GAPLINE_VERSION) { $env:GAPLINE_VERSION } else { "" }
    $InstallDir = if ($env:GAPLINE_INSTALL_DIR) { $env:GAPLINE_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "gapline\bin" }
    $NonInteractive = $env:GAPLINE_YES -eq "true"

    # --- Helpers ---
    function Write-Info  { param([string]$Msg) Write-Host "info " -ForegroundColor Blue -NoNewline; Write-Host $Msg }
    function Write-Warn  { param([string]$Msg) Write-Host "warn " -ForegroundColor Yellow -NoNewline; Write-Host $Msg }
    function Write-Err   { param([string]$Msg) Write-Host "error " -ForegroundColor Red -NoNewline; Write-Host $Msg }
    function Write-Ok    { param([string]$Msg) Write-Host "done " -ForegroundColor Green -NoNewline; Write-Host $Msg }

    # --- Architecture detection ---
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { $targetArch = "x86_64" }
        "ARM64" { $targetArch = "aarch64" }
        default { throw "Unsupported architecture: $arch" }
    }
    $target = "$targetArch-pc-windows-msvc"

    Write-Host "`nGapline CLI Installer`n" -ForegroundColor White
    Write-Info "Platform: Windows ($targetArch)"
    Write-Info "Target: $target"
    Write-Info "Install directory: $InstallDir"

    # --- Version resolution ---
    if ($Version) {
        $tag = "cli@v$Version"
        Write-Info "Using specified version: $Version"
    }
    else {
        Write-Info "Fetching latest version..."
        try {
            $releases = Invoke-RestMethod -Uri $GitHubApi -Headers @{ 'User-Agent' = 'gapline-installer' } -UseBasicParsing
            $cliRelease = $releases | Where-Object { $_.tag_name -like 'cli@*' } | Select-Object -First 1
            if (-not $cliRelease) { throw "No CLI release found" }
            $tag = $cliRelease.tag_name
        }
        catch {
            throw "Failed to fetch latest CLI release. Set `$env:GAPLINE_VERSION to install a specific version. ($_)"
        }
        $Version = $tag -replace '^cli@v', ''
        Write-Info "Latest version: $Version"
    }

    $archiveName = "$BinaryName-$target.zip"
    $downloadUrl = "$GitHubDownload/$tag/$archiveName"

    # --- Create temp directory ---
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "gapline-install-$([System.Guid]::NewGuid().ToString('N').Substring(0,8))"
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        # --- Download ---
        $archivePath = Join-Path $tmpDir $archiveName
        Write-Info "Downloading $archiveName..."
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
        }
        catch {
            throw "Download failed. Check that version $Version exists for target $target. ($_)"
        }

        # --- Checksum verification ---
        $checksumUrl = "$GitHubDownload/$tag/checksums.sha256"
        $checksumFile = Join-Path $tmpDir "checksums.sha256"
        try {
            Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumFile -UseBasicParsing 2>$null
            $checksumContent = Get-Content -Path $checksumFile
            $expectedLine = $checksumContent | Where-Object { $_ -match [regex]::Escape($archiveName) }
            if ($expectedLine) {
                $expected = ($expectedLine -split '\s+')[0]
                $actual = (Get-FileHash -Path $archivePath -Algorithm SHA256).Hash.ToLower()
                if ($actual -ne $expected) {
                    throw "Checksum verification failed! Expected: $expected, Got: $actual"
                }
                Write-Ok "Checksum verified."
            }
            else {
                Write-Warn "Archive not found in checksums file, skipping verification."
            }
        }
        catch [System.Net.WebException] {
            Write-Warn "Checksums file not available for this release, skipping verification."
        }

        # --- Extract ---
        Write-Info "Extracting..."
        Expand-Archive -Path $archivePath -DestinationPath $tmpDir -Force

        # --- Install ---
        if (-not (Test-Path -Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        $sourceBinary = Join-Path (Join-Path $tmpDir "$BinaryName-$target") "$BinaryName.exe"
        $destBinary = Join-Path $InstallDir "$BinaryName.exe"
        Copy-Item -Path $sourceBinary -Destination $destBinary -Force

        # --- Verify ---
        try {
            $installedVersion = & $destBinary --version 2>&1
            Write-Ok "Installed $installedVersion to $destBinary"
        }
        catch {
            Write-Ok "Installed gapline v$Version to $destBinary"
        }

        # --- PATH configuration ---
        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        $alreadyInPath = $false
        if ($userPath) {
            $alreadyInPath = $userPath.Split(';') -contains $InstallDir
        }

        if (-not $alreadyInPath) {
            $addPath = $true
            if (-not $NonInteractive) {
                $response = Read-Host "Add $InstallDir to PATH? [Y/n]"
                if ($response -match '^[nN]') {
                    $addPath = $false
                    Write-Warn "Skipping PATH configuration. Add it manually to your PATH."
                }
            }

            if ($addPath) {
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
        }

        Write-Host "`nRun 'gapline --help' to get started.`n"
    }
    finally {
        if (Test-Path -Path $tmpDir) {
            Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}
