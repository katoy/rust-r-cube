#[path = "src/c3x3/coord.rs"]
#[allow(dead_code, clippy::upper_case_acronyms)]
mod coord;
#[path = "src/c3x3/tables.rs"]
#[allow(dead_code)]
mod tables;
const TABLE_BYTES: Option<&[u8]> = None;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/c3x3/coord.rs");
    println!("cargo:rerun-if-changed=src/c3x3/tables.rs");
    println!("cargo:rerun-if-changed=src/c3x3/table_io.rs");
    println!("cargo:rustc-check-cfg=cfg(coverage)");
    if std::env::var("CARGO_LLVM_COV").is_ok()
        || std::env::var("RUSTFLAGS")
            .map(|f| f.contains("instrument-coverage"))
            .unwrap_or(false)
        || std::env::var("CARGO_ENCODED_RUSTFLAGS")
            .map(|f| f.contains("instrument-coverage"))
            .unwrap_or(false)
    {
        println!("cargo:rustc-cfg=coverage");
    }
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(output.join("tables.bin"), tables::encode()).unwrap();
}
