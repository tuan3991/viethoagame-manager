#!/usr/bin/env bash
# Setup MSVC environment for cargo build trên Windows
# Source file này hoặc chạy trước khi compile Rust

# VS Build Tools location
VS_TOOLS="C:\BuildTools\VC\Tools\MSVC\14.44.35207"

# Windows SDK location (lấy version mới nhất)
SDK_ROOT="C:\Program Files (x86)\Windows Kits\10"
SDK_VER=$(ls "$SDK_ROOT/Include" 2>/dev/null | sort -V | tail -1)

if [ -z "$SDK_VER" ]; then
    echo "Cannot find Windows SDK version"
    exit 1
fi

echo "Using MSVC: $VS_TOOLS"
echo "Using SDK: $SDK_VER"

# Thêm vào PATH
export PATH="$VS_TOOLS/bin/Hostx64/x64:$SDK_ROOT/bin/$SDK_VER/x64:$PATH"

# Setup INCLUDE, LIB env vars
export INCLUDE="C:\BuildTools\VC\Tools\MSVC\14.44.35207\include;$SDK_ROOT\Include\$SDK_VER\ucrt;$SDK_ROOT\Include\$SDK_VER\um;$SDK_ROOT\Include\$SDK_VER\shared;$SDK_ROOT\Include\$SDK_VER\winrt"

export LIB="C:\BuildTools\VC\Tools\MSVC\14.44.35207\lib\x64;$SDK_ROOT\Lib\$SDK_VER\ucrt\x64;$SDK_ROOT\Lib\$SDK_VER\um\x64"

echo "Environment ready for cargo build with MSVC"