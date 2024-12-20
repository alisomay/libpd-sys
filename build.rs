#![allow(dead_code)]
#![allow(unused)]

use cmake::Config;
use std::process::Command;
use std::{
    env,
    path::{Path, PathBuf},
};

// Defaults for libpd

// UTIL=true: compile utilities in libpd_wrapper/util (default)
// EXTRA=true: compile pure-data/extra externals which are then inited in libpd_init() (default)
// MULTI=true: compile with multiple instance support
// LOCALE=false: do not set the LC_NUMERIC number format to the default "C" locale* (default)

// DEBUG=true: compile with debug symbols & no optimizations
// STATIC=true: compile static library (in addition to shared library)
// FAT_LIB=true: compile universal "fat" lib with multiple architectures (macOS only)

// PORTAUDIO=true: compile with portaudio support (currently JAVA jni only)
// JAVA_HOME=/path/to/jdk: specify the path to the Java Development Kit

const PD_LOCALE: &str = "false";
const PD_MULTI: &str = "true";
const PD_UTILS: &str = "true";
const PD_EXTRA: &str = "true";
const LIBPD_RS_EXTRA: &str = "true";
const PD_FLOATSIZE: &str = "64";

#[cfg(target_os = "windows")]
/// This is needed for the GUI functions to work properly with new Pd binaries on Windows.
///
/// This will be transformed to -DWISH="\"wish86.exe\"" as a c flag
/// and will be read as "wish86.exe" in C code.
///
/// You may check the pd [source](https://github.com/pure-data/pure-data/blob/master/src/s_inter.c) to see where it is defined.
const WISH: &str = "\"\\\"wish86.exe\\\"\"";

fn main() {
    // Get the target endianness from Cargo
    let target_endian = std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();

    // Prepare the endianness defines.
    let endian_define = match target_endian.as_str() {
        "little" => vec!["-DLITTLE_ENDIAN=1234", "-DBYTE_ORDER=LITTLE_ENDIAN"],
        "big" => vec!["-DBIG_ENDIAN=4321", "-DBYTE_ORDER=BIG_ENDIAN"],
        _ => panic!("Unknown target endian: {}", target_endian),
    };

    // Directories
    let project_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let libpd_dir = project_dir.join("libpd");
    let pd_source = libpd_dir.join("pure-data").join("src");
    let libpd_wrapper_dir = libpd_dir.join("libpd_wrapper");
    let libpd_wrapper_util_dir = libpd_wrapper_dir.join("util");
    let externals_dir = libpd_wrapper_dir.join("libpd_rs_bundled").join("externals");

    // Get info about the target.
    let target_info = get_target_info();

    #[cfg(target_os = "windows")]
    {
        // For windows we need to link pthread.
        // There are some prebuilt libraries for msvc and mingw for architectures x64 and arm64.
        // Mingw support is not tested yet but should work.
        let pthread_root = project_dir.join("pthreads");

        let compiler = target_info.compiler.unwrap();

        let (pthread_lib_root, pthread_lib_path, pthread_lib_name, pthread_include): (
            PathBuf,
            PathBuf,
            &str,
            PathBuf,
        ) = match &*target_info.arch {
            "x86_64" => match compiler.as_str() {
                "msvc" => {
                    let base_dir = pthread_root
                        .join("msvc")
                        .join("pthreads_x64-windows-static");

                    let (lib_dir, lib_name) = if cfg!(debug_assertions) {
                        (base_dir.join("debug").join("lib"), "pthreadVC3d")
                    } else {
                        (base_dir.join("lib"), "pthreadVC3")
                    };

                    (
                        lib_dir.clone(),
                        lib_dir.join(format!("{}.lib", lib_name)),
                        lib_name,
                        base_dir.join("include"),
                    )
                }
                "gnu" => {
                    let lib_root = pthread_root.join("gnu/x64/lib");
                    (
                        lib_root.clone(),
                        lib_root.join("libpthreadGC2.a"),
                        "pthreadGC2",
                        pthread_root.join("gnu").join("include"),
                    )
                }
                _ => panic!("Unsupported compiler"),
            },
            "aarch64" => match compiler.as_str() {
                "msvc" => {
                    let base_dir = pthread_root
                        .join("msvc")
                        .join("pthreads_arm64-windows-static");

                    let (lib_dir, lib_name) = if cfg!(debug_assertions) {
                        (base_dir.join("debug").join("lib"), "pthreadVC3d")
                    } else {
                        (base_dir.join("lib"), "pthreadVC3")
                    };

                    (
                        lib_dir.clone(),
                        lib_dir.join(format!("{}.lib", lib_name)),
                        lib_name,
                        base_dir.join("include"),
                    )
                }
                "gnu" => {
                    let lib_root = pthread_root.join("gnu/aarch64/lib");
                    (
                        lib_root.clone(),
                        lib_root.join("libpthreadGC2.a"),
                        "pthreadGC2",
                        pthread_root.join("gnu").join("include"),
                    )
                }
                _ => panic!("Unsupported compiler"),
            },
            _ => panic!("Unsupported architecture: {}", target_info.arch),
        };

        let mut lib_destination = Config::new("libpd");

        if compiler.as_str() == "msvc" {
            // This is a complicated one..
            // So the pthreads library is built with the /MD flag for CRT.
            // One would have expected that the d suffixed prebuilt libraries would be built with /MDd flag.
            // Unfortunately this is not the case, it is not easy to rebuild the pthreads library with /MDd flag.
            // This is why we are forcing libpd to use the /MD flag even in debug mode.
            // Because mismatched CRT flags can cause linking runtime errors.
            // Another thing is that rust, if not stated otherwise, dynamic CRT flags and it is better to stay in that domain
            // since forcing /MT or /MTd flags can cause other problems.
            lib_destination.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreadedDLL");
        }

        lib_destination
            .define("PD_LOCALE", PD_LOCALE)
            .define("PD_MULTI", PD_MULTI)
            .define("PD_UTILS", PD_UTILS)
            .define("PD_EXTRA", PD_EXTRA)
            .define("LIBPD_RS_EXTRA", LIBPD_RS_EXTRA)
            .define("PTHREADS_LIB", pthread_lib_path.to_str().unwrap())
            .define("PTHREADS_INCLUDE_DIR", pthread_include.to_str().unwrap())
            .cflag(format!("-DWISH={}", WISH))
            .cflag(format!("-I{}", libpd_wrapper_dir.to_str().unwrap()))
            .cflag(format!("-I{}", pd_source.to_str().unwrap()))
            .cflag(format!("-DPD_FLOATSIZE={PD_FLOATSIZE}"))
            .no_build_target(true)
            .always_configure(true)
            .very_verbose(true);

        let lib_destination = lib_destination.build();
        let library_root = lib_destination.join("build/libs");

        // Look for pthread
        println!(
            "cargo:rustc-link-search={}",
            pthread_lib_root.to_string_lossy()
        );

        // Look for libpd
        println!("cargo:rustc-link-search={}", library_root.to_string_lossy());

        // Link pthread
        println!("cargo:rustc-link-lib=static={}", pthread_lib_name);
        // Link libpd
        if !matches!(PD_MULTI, "true") {
            println!("cargo:rustc-link-lib=static=libpd-static");
        } else {
            println!("cargo:rustc-link-lib=static=libpd-multi-static");
        }
    }

    #[cfg(target_os = "linux")]
    {
        // I love linux.. everything is concise and simple :)

        let lib_destination = Config::new("libpd")
            .define("PD_LOCALE", PD_LOCALE)
            .define("PD_MULTI", PD_MULTI)
            .define("PD_UTILS", PD_UTILS)
            .define("PD_EXTRA", PD_EXTRA)
            .define("LIBPD_RS_EXTRA", LIBPD_RS_EXTRA)
            .cflag(format!("-I{}", libpd_wrapper_dir.to_str().unwrap()))
            .cflag(format!("-I{}", pd_source.to_str().unwrap()))
            .cflag(format!("-DPD_FLOATSIZE={PD_FLOATSIZE}"))
            .no_build_target(true)
            .always_configure(true)
            .very_verbose(true)
            .build();

        let library_root = lib_destination.join("build/libs");
        println!("cargo:rustc-link-search={}", library_root.to_string_lossy());

        if !matches!(PD_MULTI, "true") {
            println!("cargo:rustc-link-lib=static=pd");
        } else {
            println!("cargo:rustc-link-lib=static=pd-multi");
        }
    }

    #[cfg(target_os = "macos")]
    {
        let lib_destination = Config::new("libpd")
            .define("PD_LOCALE", PD_LOCALE)
            .define("PD_MULTI", PD_MULTI)
            .define("PD_UTILS", PD_UTILS)
            .define("PD_EXTRA", PD_EXTRA)
            .define("LIBPD_RS_EXTRA", LIBPD_RS_EXTRA)
            .define("CMAKE_OSX_ARCHITECTURES", "x86_64;arm64")
            .cflag(format!("-I{}", libpd_wrapper_dir.to_str().unwrap()))
            .cflag(format!("-I{}", pd_source.to_str().unwrap()))
            .cflag(format!("-DPD_FLOATSIZE={PD_FLOATSIZE}"))
            .no_build_target(true)
            .always_configure(true)
            .very_verbose(true)
            .build();

        let library_root = lib_destination.join("build/libs");

        // Look for libpd
        println!("cargo:rustc-link-search={}", library_root.to_string_lossy());

        // Link libpd
        if !matches!(PD_MULTI, "true") {
            // Thins the fat library with lipo, rust linker does not like fat libs..
            thin_fat_lib(&library_root, false);
            match &*target_info.arch {
                // We now have two thin libs, one for each architecture, we need to link the appropriate one.
                // libpd-x86_64.a and libpd-aarch64.a
                "x86_64" => println!("cargo:rustc-link-lib=static=pd-x86_64"),
                "aarch64" => println!("cargo:rustc-link-lib=static=pd-aarch64"),
                _ => panic!("Unsupported architecture"),
            }
        } else {
            // Thins the fat library with lipo, rust linker does not like fat libs..
            thin_fat_lib(&library_root, true);
            match &*target_info.arch {
                // We now have two thin libs, one for each architecture, we need to link the appropriate one.
                // libpd-x86_64.a and libpd-aarch64.a
                "x86_64" => println!("cargo:rustc-link-lib=static=pd-multi-x86_64"),
                "aarch64" => println!("cargo:rustc-link-lib=static=pd-multi-aarch64"),
                _ => panic!("Unsupported architecture"),
            }
        }
    }

    // Generate bindings
    let mut bindings_builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", libpd_wrapper_dir.to_str().unwrap()))
        .clang_arg(format!("-I{}", pd_source.to_str().unwrap()))
        .clang_arg(format!("-DPD_FLOATSIZE={PD_FLOATSIZE}"))
        .clang_arg("-DPD_INTERNAL=1")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    // Add the endianness defines
    for arg in endian_define {
        bindings_builder = bindings_builder.clang_arg(arg);
    }

    #[cfg(target_os = "windows")]
    let bindings = bindings_builder
        .clang_arg(format!("-DWISH={}", WISH))
        .generate()
        .expect("Unable to generate bindings");

    #[cfg(not(target_os = "windows"))]
    let bindings = bindings_builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/// Parsed version of a target triple.
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

/// Gets info about our target.
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
fn thin_fat_lib<T: AsRef<Path>>(library_root: T, pd_multi: bool) {
    let mut name = String::from("libpd");
    if pd_multi {
        name = format!("{}-multi", name);
    }
    let root: &str = library_root.as_ref().to_str().unwrap();
    Command::new("lipo")
        .arg(format!("{root}/{name}.a"))
        .arg("-thin")
        // Apple calls aarch64, arm64
        .arg("arm64")
        .arg("-output")
        .arg(format!("{root}/{name}-aarch64.a"))
        .spawn()
        .expect("lipo command failed to start");

    Command::new("lipo")
        .arg(format!("{root}/{name}.a"))
        .arg("-thin")
        .arg("x86_64")
        .arg("-output")
        .arg(format!("{root}/{name}-x86_64.a"))
        .spawn()
        .expect("lipo command failed to start");
}
