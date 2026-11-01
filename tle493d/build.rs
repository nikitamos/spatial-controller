use std::{env, error::Error, path::PathBuf};

use glob::glob;

pub fn main() -> Result<(), Box<dyn Error>> {
    generate_bindings()?;
    compile_c()?;

    Ok(())
}

fn compile_c() -> Result<(), Box<dyn Error>> {
    let sources = glob("infineon-lib/src/corelib/*.c")?.map(|x| x.expect("failed to glob sources"));
    cc::Build::new()
        .emit_rerun_if_env_changed(true)
        .include("infineon-lib/src")
        .files(sources)
        .compile("tle493d-cc");
    Ok(())
}

fn generate_bindings() -> Result<(), Box<dyn Error + 'static>> {
    let headers = glob("infineon-lib/src/corelib/*.h")?.map(|x| {
        x.expect("failed to glob headers")
            .to_str()
            .map(ToString::to_string)
            .expect("bad path")
    });
    let bindings = bindgen::builder()
        .headers(headers)
        .use_core()
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .clang_arg("-Iinfineon-lib/src")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?;
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings.write_to_file(out_path)?;
    Ok(())
}
