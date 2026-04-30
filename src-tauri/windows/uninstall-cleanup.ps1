$ErrorActionPreference = "Continue"

function Add-PathIfPresent {
  param(
    [System.Collections.Generic.List[string]]$Paths,
    [string]$Path
  )

  if ([string]::IsNullOrWhiteSpace($Path)) {
    return
  }

  $Paths.Add($Path)
}

function Add-ConfiguredPath {
  param(
    [System.Collections.Generic.List[string]]$Paths,
    [string]$Path,
    [string]$RelativeBase
  )

  if ([string]::IsNullOrWhiteSpace($Path)) {
    return
  }

  if ([System.IO.Path]::IsPathRooted($Path)) {
    $Paths.Add($Path)
    return
  }

  if (-not [string]::IsNullOrWhiteSpace($RelativeBase)) {
    $Paths.Add((Join-Path $RelativeBase $Path))
  }
}

function Test-SafeRemovalPath {
  param([string]$Path)

  if ([string]::IsNullOrWhiteSpace($Path)) {
    return $false
  }

  try {
    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $root = [System.IO.Path]::GetPathRoot($fullPath)
  } catch {
    return $false
  }

  if ($fullPath -eq $root) {
    return $false
  }

  $blocked = @(
    $env:USERPROFILE,
    $env:APPDATA,
    $env:LOCALAPPDATA,
    $env:ProgramFiles,
    ${env:ProgramFiles(x86)},
    $env:WINDIR
  ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

  foreach ($blockedPath in $blocked) {
    try {
      if ([System.IO.Path]::GetFullPath($blockedPath).TrimEnd('\') -ieq $fullPath.TrimEnd('\')) {
        return $false
      }
    } catch {
    }
  }

  return $true
}

$roamingAppDir = Join-Path $env:APPDATA "com.smart.autodaily"
$localAppDir = Join-Path $env:LOCALAPPDATA "com.smart.autodaily"
$legacyRoamingDir = Join-Path $env:APPDATA "auto_daily"
$legacyLocalDir = Join-Path $env:LOCALAPPDATA "auto_daily"
$storePath = Join-Path $roamingAppDir "autodaily.config.json"
$paths = [System.Collections.Generic.List[string]]::new()

Add-PathIfPresent $paths (Join-Path $roamingAppDir "autodaily.db")
Add-PathIfPresent $paths (Join-Path $roamingAppDir "autodaily.db-shm")
Add-PathIfPresent $paths (Join-Path $roamingAppDir "autodaily.db-wal")
Add-PathIfPresent $paths (Join-Path $roamingAppDir "scripts")
Add-PathIfPresent $paths (Join-Path $roamingAppDir "ocr-text-cache")
Add-PathIfPresent $paths (Join-Path $localAppDir "logs")

if (Test-Path -LiteralPath $storePath) {
  try {
    $config = Get-Content -LiteralPath $storePath -Raw | ConvertFrom-Json

    if ($null -ne $config.scripts_config -and $null -ne $config.scripts_config.dir) {
      Add-ConfiguredPath $paths ([string]$config.scripts_config.dir) $roamingAppDir
    }

    if ($null -ne $config.log_config -and $null -ne $config.log_config.logDir) {
      Add-ConfiguredPath $paths ([string]$config.log_config.logDir) $localAppDir
    }

    if ($null -ne $config.vision_text_cache_config) {
      $cacheDir = [string]$config.vision_text_cache_config.dir
      if ([string]::IsNullOrWhiteSpace($cacheDir)) {
        Add-PathIfPresent $paths (Join-Path $roamingAppDir "ocr-text-cache")
      } else {
        Add-ConfiguredPath $paths $cacheDir $roamingAppDir
      }
    }
  } catch {
    Write-Host "Failed to parse AutoDaily config: $($_.Exception.Message)"
  }
}

Add-PathIfPresent $paths $roamingAppDir
Add-PathIfPresent $paths $localAppDir
Add-PathIfPresent $paths $legacyRoamingDir
Add-PathIfPresent $paths $legacyLocalDir

$paths |
  Select-Object -Unique |
  Where-Object { Test-SafeRemovalPath $_ } |
  Sort-Object Length -Descending |
  ForEach-Object {
    if (Test-Path -LiteralPath $_) {
      Write-Host "Removing AutoDaily user data: $_"
      Remove-Item -LiteralPath $_ -Recurse -Force -ErrorAction Continue
    }
  }
