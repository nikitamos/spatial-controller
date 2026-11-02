use std::{collections::HashSet, env, error::Error, path::PathBuf};

use glob::glob;

pub fn main() -> Result<(), Box<dyn Error>> {
    let features = env::var("CARGO_CFG_FEATURE").unwrap_or_default();
    let features: HashSet<&str> = features.split(',').collect();
    generate_bindings(&features)?;
    compile_c(&features)?;

    Ok(())
}

fn compile_c(features: &HashSet<&str>) -> Result<(), Box<dyn Error>> {
    let sources = glob("infineon-lib/src/corelib/*.c")?.map(|x| x.expect("failed to glob sources"));
    let mut builder = cc::Build::new();
    builder
        .emit_rerun_if_env_changed(true)
        .include("infineon-lib/src")
        .files(sources);
    if features.contains("use-f64") {
        builder.define("TLX493D_USE_DOUBLES", None);
    }
    builder.compile("tle493d-cc");
    Ok(())
}

fn generate_bindings(features: &HashSet<&str>) -> Result<(), Box<dyn Error + 'static>> {
    let headers = glob("infineon-lib/src/corelib/*.h")?.map(|x| {
        x.expect("failed to glob headers")
            .to_str()
            .map(ToString::to_string)
            .expect("bad path")
    });
    let mut builder = bindgen::builder()
        .headers(headers)
        .use_core()
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .clang_arg("-Iinfineon-lib/src")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    if features.contains("use-f64") {
        builder = builder.clang_arg("-DTLX493D_USE_DOUBLES");
    }
    let bindings = builder.generate()?;
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings.write_to_file(out_path)?;
    Ok(())
}
