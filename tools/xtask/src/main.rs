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
    if !header.contains("#define GWT_ABI_VERSION 1u") {
        bail!("C header ABI version does not match the Rust ABI version");
    }
    println!("C ABI header is synchronized");
    Ok(())
}
