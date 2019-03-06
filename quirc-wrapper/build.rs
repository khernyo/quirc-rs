use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), io::Error> {
    let quirc_path = Path::new("quirc");
    let quirc_src_path = quirc_path.join("lib");
    let lib_filename = "libquirc.a";

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    Command::new("make").current_dir(quirc_path).arg(lib_filename).status()?;
    fs::copy(quirc_path.join(lib_filename), out_path.join(lib_filename))?;
    Command::new("make").current_dir(quirc_path).arg("clean").status()?;

    println!("cargo:rustc-link-lib=static=quirc");
    println!("cargo:rustc-link-search=native={}", out_dir);

    let bindings = bindgen::Builder::default()
        .clang_args(&["-I", quirc_src_path.to_str().unwrap()])
        .derive_debug(true)
        .derive_eq(true)
        .derive_partialeq(true)
        .impl_debug(true)
        .impl_partialeq(true)
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
