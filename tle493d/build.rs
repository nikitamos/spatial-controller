use std::{env, path::PathBuf};

use glob::glob;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let headers = glob("infineon-lib/src/corelib/*.h")?.map(|x| {
        x.expect("failed to glob headers")
            .to_str()
            .map(ToString::to_string)
            .expect("bad path")
    });
    let bindings = bindgen::builder()
        .headers(headers)
        .use_core()
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .clang_arg("-Iinfineon-lib/src")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings.write_to_file(out_path)?;

    Ok(())
}
