// Generate bindings for isl. Based on https://rust-lang.github.io/rust-bindgen/tutorial-3.html

use std::env;
use std::path::PathBuf;

fn main() {
    let isl_prefix = match env::var("ISL_PREFIX") {
        Ok(p) => PathBuf::from(p),
        Err(_) => PathBuf::from("/usr"),
    };
    println!("cargo:rustc-link-search={}", isl_prefix.join("lib").display());
    println!("cargo:rustc-link-lib=isl");

    let bindings = bindgen::Builder::default()
        .header("isl_wrapper.h")
        .clang_arg(format!("-I{}", isl_prefix.join("include").display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("isl_bindings.rs"))
        .expect("Couldn't write bindings!");
}
