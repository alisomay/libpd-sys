use cmake::Config;
use std::process::Command;
use std::{env, path::PathBuf};

#[derive(Debug)]
struct Triple {
    arch: String,
    vendor: String,
    os: String,
}
impl From<Vec<&str>> for Triple {
    fn from(triple: Vec<&str>) -> Self {
        Triple {
            arch: triple[0].to_string(),
            vendor: triple[1].to_string(),
            os: triple[2].to_string(),
        }
    }
}

fn get_host_triple() -> Triple {
    let triple = std::env::var("TARGET").unwrap();
    let triple: Vec<&str> = triple.split('-').collect();
    Triple::from(triple)
}

/// Thins the fat library with lipo here, rust linker does not like fat libs..
///
/// ```sh
/// lipo libpd.a -thin arm64 -output libpd-aarch64.a
/// lipo libpd.a -thin x86_64 -output libpd-x86_64.a
/// ```
fn thin_fat_lib(library_root: &str) {
    Command::new("lipo")
        .arg(format!("{library_root}/libpd.a"))
        .arg("-thin")
        // Apple calls aarch64, arm64
        .arg("arm64")
        .arg("-output")
        .arg(format!("{library_root}/libpd-aarch64.a"))
        .spawn()
        .expect("lipo command failed to start");

    Command::new("lipo")
        .arg(format!("{library_root}/libpd.a"))
        .arg("-thin")
        .arg("x86_64")
        .arg("-output")
        .arg(format!("{library_root}/libpd-x86_64.a"))
        .spawn()
        .expect("lipo command failed to start");
}

fn main() {
    // TODO: Make pd compilation settings configurable.
    let host_triple = get_host_triple();


    #[cfg(target_os = "windows")]
    let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    #[cfg(target_os = "windows")]
    let pthread_include = format!("{project_dir}/pthread/Pre-built.2/include");
    #[cfg(target_os = "windows")]
    let pthread_lib_root = format!("{project_dir}/pthread/Pre-built.2/lib");
    
    // This is incomplete
    // Check if cargo can get correct arch, x86 or x64
    // Link the library accordingly
    // I assume that this will work in x64 windows but I don't have a machine to test that
    // I know that x64 or x84 does not work on ARM, I need to either correctly build or find a 
    // built version for ARM.
    #[cfg(target_os = "windows")]
    let pthread_lib = match &*host_triple.arch {
        // host_triple returns "x86_64", how to determine the right pthread lib?
        // lets try defaulting to x64
        "x86_64" => "/x64/pthreadVC2.lib",
        _ => panic!()
    };
    
    #[cfg(target_os = "windows")]
    let pthread_lib = format!("{pthread_lib_root}{pthread_lib}");

    #[cfg(target_os = "windows")]
    let lib_destination = Config::new("libpd")
        .define("PD_EXTRA", "true")
        .define("PD_LOCALE", "false")
        // Without multi instance support
        .define("PD_MULTI", "false")
        .define("PD_UTILS", "true")
        .define("CMAKE_THREAD_LIBS_INIT", pthread_lib)
        .define("PTHREADS_INCLUDE_DIR", pthread_include)
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();

    #[cfg(target_os = "linux")]
    let lib_destination = Config::new("libpd")
        .define("PD_EXTRA", "true")
        .define("PD_LOCALE", "false")
        // Without multi instance support
        .define("PD_MULTI", "false")
        .define("PD_UTILS", "true")
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();

    #[cfg(target_os = "macos")]
    let lib_destination = Config::new("libpd")
        .define("PD_EXTRA", "true")
        .define("PD_LOCALE", "false")
        // Without multi instance support
        .define("PD_MULTI", "false")
        .define("PD_UTILS", "true")
        .define("CMAKE_OSX_ARCHITECTURES", "x86_64;arm64")
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();

    let library_root = format!("{}/build/libs", lib_destination.as_path().display());

    #[cfg(target_os = "macos")]
    thin_fat_lib(&library_root);
    // We now have two thin libs, one for each architecture, we need to link the appropriate one.
    // libpd-x86_64.a and libpd-aarch64.a

    println!("cargo:rustc-link-search={library_root}");

    #[cfg(target_os = "macos")]
    match &*host_triple.arch {
        "x86_64" => println!("cargo:rustc-link-lib=static=pd-x86_64"),
        "aarch64" => println!("cargo:rustc-link-lib=static=pd-aarch64"),
        _ => panic!("Unsupported architecture"),
    }

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=static=pd");

    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=static=libpd-static");


    // Generate bindings
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
