//! Embeds the Windows icon, version info and manifest (assets/app.res).
//!
//! `.res` files are understood by LLD (used by build.ps1) and by MSVC's
//! link.exe, but not by GNU ld, so the resource is only linked for those.

fn main() {
    println!("cargo:rerun-if-changed=assets/app.res");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let flags = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    let uses_lld = flags.contains("lld");

    if target_env == "msvc" || uses_lld {
        let dir = std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir");
        let res = std::path::Path::new(&dir).join("assets").join("app.res");
        println!("cargo:rustc-link-arg-bins={}", res.display());
    } else {
        println!("cargo:warning=Linker cannot embed assets/app.res (icon/version info); build with build.ps1 or the MSVC toolchain.");
    }
}
