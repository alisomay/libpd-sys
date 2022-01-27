use cmake::Config;
use std::{env, path::PathBuf};

fn main() {
    let lib_destination = Config::new("libpd")
        .define("PD_EXTRA", "true")
        .define("PD_LOCALE", "false")
        // Without multi instance support
        .define("PD_MULTI", "false")
        .define("PD_UTILS", "true")
        .define("CMAKE_OSX_ARCHITECTURES", "arm64")
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();
    println!(
        "cargo:rustc-link-search={}/build/libs",
        lib_destination.as_path().display()
    );
    println!("cargo:rustc-link-lib=static={}", "pd");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .rustfmt_bindings(true)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
