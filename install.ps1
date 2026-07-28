$ErrorActionPreference = "Stop"

$repositoryUrl = "https://github.com/Captain-AI-Hub/WeepCode"
$architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()

if (-not $IsWindows -and $PSVersionTable.PSEdition -eq "Core") {
    throw "install.ps1 supports Windows only."
}
if ($architecture -ne "X64") {
    throw "WeepCode currently supports Windows only on x86_64. Detected: $architecture"
}

$assetName = "weepcode-windows-x86_64.zip"
if ($env:WEEPCODE_VERSION) {
    $downloadBaseUrl = "$repositoryUrl/releases/download/$($env:WEEPCODE_VERSION)"
} else {
    $downloadBaseUrl = "$repositoryUrl/releases/latest/download"
}

if ($env:WEEPCODE_INSTALL_DIR) {
    $installDirectory = $env:WEEPCODE_INSTALL_DIR
} else {
    $localApplicationData = [Environment]::GetFolderPath("LocalApplicationData")
    $installDirectory = Join-Path $localApplicationData "Programs\WeepCode\bin"
}

$temporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ("weepcode-install-" + [guid]::NewGuid())
$archivePath = Join-Path $temporaryDirectory $assetName
$checksumsPath = Join-Path $temporaryDirectory "SHA256SUMS"
$extractionDirectory = Join-Path $temporaryDirectory "extracted"

New-Item -ItemType Directory -Force -Path $temporaryDirectory | Out-Null

try {
    Invoke-WebRequest -UseBasicParsing -Uri "$downloadBaseUrl/$assetName" -OutFile $archivePath
    Invoke-WebRequest -UseBasicParsing -Uri "$downloadBaseUrl/SHA256SUMS" -OutFile $checksumsPath

    $escapedAssetName = [regex]::Escape($assetName)
    $checksumLine = Get-Content $checksumsPath |
        Where-Object { $_ -match "\s+$escapedAssetName$" } |
        Select-Object -First 1
    if (-not $checksumLine) {
        throw "No checksum was published for $assetName."
    }

    $expectedChecksum = ($checksumLine -split "\s+")[0].ToLowerInvariant()
    $actualChecksum = (Get-FileHash -Algorithm SHA256 $archivePath).Hash.ToLowerInvariant()
    if ($actualChecksum -ne $expectedChecksum) {
        throw "Checksum verification failed for $assetName."
    }

    New-Item -ItemType Directory -Force -Path $extractionDirectory | Out-Null
    Expand-Archive -Path $archivePath -DestinationPath $extractionDirectory -Force
    New-Item -ItemType Directory -Force -Path $installDirectory | Out-Null
    $sourceExecutablePath = Join-Path $extractionDirectory "weepcode.exe"
    $installedExecutablePath = Join-Path $installDirectory "weepcode.exe"
    Copy-Item -Path $sourceExecutablePath -Destination $installedExecutablePath -Force

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathEntries = @($userPath -split ";" | Where-Object { $_ })
    if ($pathEntries -notcontains $installDirectory) {
        $updatedUserPath = (@($pathEntries) + $installDirectory) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $updatedUserPath, "User")
    }
    if (($env:Path -split ";") -notcontains $installDirectory) {
        $env:Path = "$installDirectory;$env:Path"
    }

    Write-Host "WeepCode installed to $installedExecutablePath"
} finally {
    Remove-Item -Recurse -Force $temporaryDirectory -ErrorAction SilentlyContinue
}
