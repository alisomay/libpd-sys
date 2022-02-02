use cmake::Config;
use std::process::Command;
use std::{env, path::PathBuf};

// TODO: Make pd compilation settings configurable from cargo.
const PD_EXTRA: &str = "true";
const PD_LOCALE: &str = "false";
const PD_UTILS: &str = "true";

fn main() {
    let mut pd_multi = "false";

    if cfg!(feature = "multi") {
        pd_multi = "true";
    }

    let target_info = get_target_info();

    #[cfg(target_os = "windows")]
    let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    #[cfg(target_os = "windows")]
    let pthread_include = format!("{project_dir}/pthread/Pre-built.2/include");
    #[cfg(target_os = "windows")]
    let pthread_lib_root = format!("{project_dir}/pthread/Pre-built.2/lib");
    #[cfg(target_os = "windows")]
    let pthread_lib = match &*target_info.arch {
        // These two should work but haven't been tested yet
        "x86_64" => match &*(target_info.compiler.unwrap()) {
            "msvc" => "/x64/pthreadVC2.lib",
            "gnu" => "/x64/libpthreadGC2.a",
            _ => panic!("Unsupported compiler"),
        },
        "i686" => match &*(target_info.compiler.unwrap()) {
            "msvc" => "/x86/pthreadVC2.lib",
            "gnu" => "/x86/libpthreadGC2.a",
            _ => panic!("Unsupported compiler"),
        },
        // This is experimental, I've built this in a Windows 11 Arm VM, haven't tested yet
        // I don't know what do here yet. Maybe I need to build one with msvc also?
        "aarch64" => "/aarch64/libpthreadGC2.a",
        //
        _ => panic!("Unsupported architecture"),
    };
    #[cfg(target_os = "windows")]
    let pthread_lib = format!("{pthread_lib_root}{pthread_lib}");

    #[cfg(target_os = "windows")]
    let lib_destination = Config::new("libpd")
        .define("PD_EXTRA", PD_EXTRA)
        .define("PD_LOCALE", PD_LOCALE)
        .define("PD_MULTI", pd_multi)
        .define("PD_UTILS", PD_UTILS)
        .define("CMAKE_THREAD_LIBS_INIT", pthread_lib)
        .define("PTHREADS_INCLUDE_DIR", pthread_include)
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();

    #[cfg(target_os = "linux")]
    let lib_destination = Config::new("libpd")
        .define("PD_EXTRA", PD_EXTRA)
        .define("PD_LOCALE", PD_LOCALE)
        .define("PD_MULTI", pd_multi)
        .define("PD_UTILS", PD_UTILS)
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();

    #[cfg(target_os = "macos")]
    let lib_destination = Config::new("libpd")
        .define("PD_EXTRA", PD_EXTRA)
        .define("PD_LOCALE", PD_LOCALE)
        .define("PD_MULTI", pd_multi)
        .define("PD_UTILS", PD_UTILS)
        .define("CMAKE_OSX_ARCHITECTURES", "x86_64;arm64")
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();

    let library_root = format!("{}/build/libs", lib_destination.as_path().display());
    println!("cargo:rustc-link-search={library_root}");

    #[cfg(target_os = "macos")]
    thin_fat_lib(&library_root);

    #[cfg(target_os = "macos")]
    match &*target_info.arch {
        // We now have two thin libs, one for each architecture, we need to link the appropriate one.
        // libpd-x86_64.a and libpd-aarch64.a
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

#[allow(dead_code)]
#[derive(Debug)]
struct TargetInfo {
    arch: String,
    vendor: String,
    os: String,
    compiler: Option<String>,
}
impl From<Vec<&str>> for TargetInfo {
    fn from(info: Vec<&str>) -> Self {
        TargetInfo {
            arch: info[0].to_string(),
            vendor: info[1].to_string(),
            os: info[2].to_string(),
            compiler: info
                .get(3)
                .map_or_else(|| None, |value| Some(value.to_owned().to_owned())),
        }
    }
}

fn get_target_info() -> TargetInfo {
    let info = std::env::var("TARGET").unwrap();
    let info: Vec<&str> = info.split('-').collect();
    TargetInfo::from(info)
}

/// Thins the fat library with lipo, rust linker does not like fat libs..
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
