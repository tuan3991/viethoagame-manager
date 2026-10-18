# Setup MSVC environment for cargo on Windows PowerShell
# Usage: . .\scripts\setup-msvc-env.ps1

$VS_TOOLS = "C:\BuildTools\VC\Tools\MSVC\14.44.35207"
$SDK_ROOT = "C:\Program Files (x86)\Windows Kits\10"

# Find latest SDK version
$SDK_VERSIONS = Get-ChildItem "$SDK_ROOT\Include" -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending
$SDK_VER = $SDK_VERSIONS[0].Name

Write-Host "Using MSVC: $VS_TOOLS" -ForegroundColor Green
Write-Host "Using SDK:  $SDK_VER" -ForegroundColor Green

# Add to PATH
$env:Path = "$VS_TOOLS\bin\Hostx64\x64;$SDK_ROOT\bin\$SDK_VER\x64;$env:USERPROFILE\.cargo\bin;$env:Path"

# INCLUDE
$env:INCLUDE = "$VS_TOOLS\include;$SDK_ROOT\Include\$SDK_VER\ucrt;$SDK_ROOT\Include\$SDK_VER\um;$SDK_ROOT\Include\$SDK_VER\shared;$SDK_ROOT\Include\$SDK_VER\winrt"

# LIB
$env:LIB = "$VS_TOOLS\lib\x64;$SDK_ROOT\Lib\$SDK_VER\ucrt\x64;$SDK_ROOT\Lib\$SDK_VER\um\x64"

Write-Host "MSVC environment ready. Try: cargo check" -ForegroundColor Green