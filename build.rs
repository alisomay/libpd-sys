use std::{env, path::PathBuf};

use cmake::Config;

fn main() {
    let lib_destination = Config::new("libpd")
        .define("PD_EXTRA", "true")
        .define("PD_LOCALE", "false")
        .define("PD_MULTI", "false")
        .define("PD_UTILS", "true")
        // Win only
        // When using Microsoft Visual Studio (MSVC), you will be requested to provide a path to the pthreads library and its headers using variables CMAKE_THREAD_LIBS_INIT and PTHREADS_INCLUDE_DIR.
        // OSX only
        .define("CMAKE_OSX_ARCHITECTURES", "arm64")
        // .define("CMAKE_OSX_DEPLOYMENT_TARGET","")
        // .cflag("-foo")
        // Check this
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();

    // dbg!("HEY");
    // dbg!(lib_destination.as_path().display());
    // panic!();
    println!(
        "cargo:rustc-link-search={}/build/libs",
        lib_destination.as_path().display()
    );
    println!("cargo:rustc-link-lib=static={}", "pd");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .rustfmt_bindings(true)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        // .blocklist_type("t_libpd_printhook")
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libpd_bindings.rs"))
        .expect("Couldn't write bindings!");
}
