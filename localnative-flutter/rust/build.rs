fn main() {
    // Build script for flutter_rust_bridge
    // The code generation happens via flutter_rust_bridge_codegen CLI tool
    println!("cargo:rerun-if-changed=src/lib.rs");
}
