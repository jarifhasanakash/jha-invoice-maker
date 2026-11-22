# Builds "release\JHA Invoice Maker.exe" — no Visual Studio required.
#
# Usage (PowerShell, from this folder):
#   .\build.ps1            # release build -> release\JHA Invoice Maker.exe
#   .\build.ps1 -Run       # build and start the app
#   .\build.ps1 -Test      # run the unit tests
#
# How it works: this machine has Rust's GNU toolchain but no C toolchain.
# Rust links with its bundled rust-lld, and import libraries are generated
# with Zig's llvm-dlltool (installed once into .tools\ via pip, ~70 MB).
# If you install "Visual Studio Build Tools (C++)" and switch to the MSVC
# toolchain (`rustup default stable-x86_64-pc-windows-msvc`), a plain
# `cargo build --release` works too and this script is not needed.

param([switch]$Run, [switch]$Test)
# Native tools (pip, cargo) write progress to stderr, so check exit codes instead.
$ErrorActionPreference = 'Continue'

$root  = $PSScriptRoot
$tools = Join-Path $root '.tools'
$venv  = Join-Path $tools 'venv'
$zig   = Join-Path $venv 'Lib\site-packages\ziglang\zig.exe'
$shim  = Join-Path $tools 'dlltool.exe'

New-Item -ItemType Directory -Force $tools | Out-Null

if (-not (Test-Path $zig)) {
    Write-Host 'Setting up build tools (one time): installing Zig into .tools ...'
    python -m venv $venv
    if ($LASTEXITCODE -ne 0) { throw 'Python 3 is required to install the build tools.' }
    & (Join-Path $venv 'Scripts\python.exe') -m pip install --quiet --disable-pip-version-check ziglang
    if ($LASTEXITCODE -ne 0) { throw 'Could not install ziglang with pip.' }
}

if (-not (Test-Path $shim)) {
    rustup run stable-x86_64-pc-windows-gnu rustc -O (Join-Path $root 'tools\dlltool-shim.rs') -o $shim
    if ($LASTEXITCODE -ne 0) { throw 'Could not compile the dlltool shim.' }
}

$sep = [char]0x1f
$env:JHA_ZIG = $zig
$env:CARGO_ENCODED_RUSTFLAGS = @('-C', "dlltool=$shim", '-C', 'linker=rust-lld', '-C', 'linker-flavor=ld.lld') -join $sep

Push-Location $root
try {
    if ($Test) {
        cargo +stable-x86_64-pc-windows-gnu test
        if ($LASTEXITCODE -ne 0) { throw 'Tests failed.' }
        return
    }
    cargo +stable-x86_64-pc-windows-gnu build --release
    if ($LASTEXITCODE -ne 0) { throw 'Build failed.' }

    # (dist\ belongs to the old Electron builder, so the Rust build goes to release\)
    $outDir = Join-Path $root 'release'
    New-Item -ItemType Directory -Force $outDir | Out-Null
    $out = Join-Path $outDir 'JHA Invoice Maker.exe'
    Copy-Item (Join-Path $root 'target\release\invoice_maker.exe') $out -Force
    $mb = [math]::Round((Get-Item $out).Length / 1MB, 1)
    Write-Host "Built $out ($mb MB)"
    if ($Run) { Start-Process $out }
}
finally {
    Pop-Location
}
