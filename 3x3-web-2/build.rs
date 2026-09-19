#[path = "src/coord.rs"]
#[allow(dead_code, clippy::upper_case_acronyms)]
mod coord;
#[path = "src/tables.rs"]
#[allow(dead_code)]
mod tables;
const TABLE_BYTES: Option<&[u8]> = None;

fn main() {
    println!("cargo:rerun-if-changed=src/coord.rs");
    println!("cargo:rerun-if-changed=src/tables.rs");
    println!("cargo:rerun-if-changed=src/table_io.rs");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(output.join("tables.bin"), tables::encode()).unwrap();
}
