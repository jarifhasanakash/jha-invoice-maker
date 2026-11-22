# Builds the web version's PDF engine (Rust -> WebAssembly) and copies it to
# ..\pdf-engine.wasm, next to index.html.
#
#   .\build.ps1
#
# Needs Rust (stable) with the wasm32-unknown-unknown target:
#   rustup target add wasm32-unknown-unknown
#
# The engine shares its source with the desktop app (..\desktop\src). On a PC
# without Visual Studio, run ..\desktop\build.ps1 once first: it installs the
# linker helpers in ..\desktop\.tools that build scripts need here as well.

$ErrorActionPreference = 'Continue'
$root = $PSScriptRoot
$desktopTools = Join-Path $root '..\desktop\.tools'
$zig = Join-Path $desktopTools 'venv\Lib\site-packages\ziglang\zig.exe'

$toolchain = @()
if (Test-Path $zig) {
    # GNU toolchain + rust-lld for host build scripts (same setup as the desktop build).
    $env:JHA_ZIG = $zig
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = 'rust-lld'
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = '-C linker-flavor=ld.lld'
    $toolchain = @('+stable-x86_64-pc-windows-gnu')
}
Remove-Item Env:CARGO_ENCODED_RUSTFLAGS -ErrorAction SilentlyContinue

Push-Location $root
try {
    cargo @toolchain build --release --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { throw 'Build failed.' }
    $out = Join-Path $root '..\pdf-engine.wasm'
    Copy-Item (Join-Path $root 'target\wasm32-unknown-unknown\release\jha_invoice_engine.wasm') $out -Force
    $mb = [math]::Round((Get-Item $out).Length / 1MB, 1)
    Write-Host "Built $((Resolve-Path $out).Path) ($mb MB)"
}
finally {
    Pop-Location
}
