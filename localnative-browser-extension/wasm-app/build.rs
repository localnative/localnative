use fluent_static_codegen::FunctionPerMessageCodeGenerator;
pub fn main() {
    if let Ok(src) =
        fluent_static_codegen::generate("./locales/", FunctionPerMessageCodeGenerator::new("en"))
    {
        let destination = ::std::path::Path::new("src").join("locales.rs");
        ::std::fs::write(&destination, src).expect("Error writing generated sources");
        let output = ::std::process::Command::new("rustfmt")
            .arg(&destination)
            .output()
            .expect("Failed to run rustfmt");
        if !output.status.success() {
            panic!("Failed to format generated sources");
        }
    } else {
        panic!("Failed to generate locale sources");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
