//! Tiny `dlltool` replacement used by build.ps1.
//!
//! The GNU Rust toolchain needs `dlltool` to create import libraries for the
//! `windows` crates, but the copy bundled with rustup requires a GNU assembler
//! that isn't installed. This forwards rustc's call to Zig's bundled
//! `llvm-dlltool` (path in the `JHA_ZIG` environment variable), dropping the
//! one GNU-only flag (`--temp-prefix`) that llvm-dlltool doesn't accept.

use std::process::{Command, exit};

fn main() {
    let zig = std::env::var("JHA_ZIG").unwrap_or_else(|_| {
        eprintln!("dlltool-shim: JHA_ZIG is not set (run build.ps1)");
        exit(2)
    });
    let mut args = Vec::new();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == "--temp-prefix" {
            it.next();
            continue;
        }
        args.push(a);
    }
    let status = Command::new(zig)
        .arg("dlltool")
        .args(&args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("dlltool-shim: cannot run zig: {e}");
            exit(2)
        });
    exit(status.code().unwrap_or(1));
}
