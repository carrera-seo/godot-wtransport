use std::path::Path;

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "check-header".into());
    match command.as_str() {
        "check-header" => check_header(),
        _ => bail!("unknown xtask command: {command}"),
    }
}

fn check_header() -> Result<()> {
    let header = std::fs::read_to_string(Path::new("rust/ffi/include/godot_wtransport.h"))
        .context("read C ABI header")?;
    let rust = std::fs::read_to_string(Path::new("rust/ffi/src/lib.rs"))
        .context("read Rust FFI source")?;
    let version = rust
        .lines()
        .find_map(|line| {
            line.strip_prefix("pub const ABI_VERSION: u32 = ")
                .and_then(|value| value.strip_suffix(';'))
        })
        .context("find Rust ABI version")?;
    if !header.contains(&format!("#define GWT_ABI_VERSION {version}u")) {
        bail!("C header ABI version does not match the Rust ABI version");
    }
    println!("C ABI header is synchronized");
    Ok(())
}
