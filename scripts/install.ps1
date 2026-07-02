#!/usr/bin/env pwsh
$ErrorActionPreference = 'Stop'

$Repo = 'fkm-X3/idot-lang'
$InstallDir = if ($env:IDOT_HOME) { $env:IDOT_HOME } else { Join-Path $HOME '.idot' }
$BinDir = Join-Path $InstallDir 'bin'
$Version = if ($env:IDOT_VERSION) { $env:IDOT_VERSION } else { 'latest' }

function Say($msg) { Write-Host "==> $msg" -ForegroundColor Green }
function Err($msg) { Write-Host "==> $msg" -ForegroundColor Red; throw $msg }

function Detect-Platform {
    $raw = switch -regex ($env:PROCESSOR_ARCHITECTURE) {
        'AMD64|x86_64' { 'x86_64' }
        'ARM64|aarch64' { 'aarch64' }
        'X86|x86' { Err "32-bit Windows is not supported by idot-lang; use a 64-bit OS" }
        default { Err "unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
    }
    "windows-$raw"
}

function Get-DownloadUrl {
    param([string]$Platform)

    if ($Version -eq 'latest') {
        $api = "https://api.github.com/repos/$Repo/releases/latest"
        try {
            $tag = (Invoke-RestMethod -Uri $api -UseBasicParsing).tag_name
        } catch {
            Err "could not determine latest release"
        }
    } else {
        $tag = $Version
    }

    $ext = if ($Platform -like 'windows-*') { 'zip' } else { 'tar.gz' }
    "https://github.com/$Repo/releases/download/$tag/idot-$tag-$Platform.$ext"
}

function Install-Binaries {
    param([string]$Url)

    $tmp = Join-Path $env:TEMP "idot-install-$([System.Guid]::NewGuid())"
    New-Item -ItemType Directory -Path $tmp -Force | Out-Null
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null

    $ext = if ($Url -like '*.zip') { 'zip' } else { 'tar.gz' }
    $archive = Join-Path $tmp "idot-archive.$ext"
    Say "downloading $Url"
    Invoke-WebRequest -Uri $Url -OutFile $archive -UseBasicParsing

    if ($ext -eq 'zip') {
        Expand-Archive -Path $archive -DestinationPath $tmp -Force
    } else {
        tar xzf $archive -C $tmp
    }

    $idotPath = Join-Path $BinDir 'idot.exe'
    $matrixPath = Join-Path $BinDir 'matrix.exe'

    if (Test-Path (Join-Path $tmp 'idot.exe')) {
        Copy-Item (Join-Path $tmp 'idot.exe') $idotPath -Force
    }
    if (Test-Path (Join-Path $tmp 'matrix.exe')) {
        Copy-Item (Join-Path $tmp 'matrix.exe') $matrixPath -Force
    }

    Remove-Item -Path $tmp -Recurse -Force
    Say "installed to $BinDir"
}

function Update-Path {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($userPath -notlike "*$BinDir*") {
        $newPath = "$BinDir;$userPath"
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        Say "added $BinDir to user PATH"
    }
}

function Main {
    $platform = Detect-Platform
    $url = Get-DownloadUrl $platform
    Install-Binaries $url
    if ($env:IDOT_NO_PATH_UPDATE -ne '1') { Update-Path }
    Say "done! restart your terminal or run:`$env:Path = `"$BinDir;`$env:Path`""
}

Main
