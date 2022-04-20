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

const PD_EXTRA: &str = "true";
const PD_LOCALE: &str = "false";
const PD_UTILS: &str = "true";
const PD_FLOATSIZE: &str = "64";

fn main() {
    // Directories
    let project_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let libpd_dir = project_dir.join("libpd");
    let pd_source = libpd_dir.join("pure-data").join("src");
    let libpd_wrapper_dir = libpd_dir.join("libpd_wrapper");
    let libpd_wrapper_util_dir = libpd_wrapper_dir.join("util");

    // Transform values of the #include fields in libpd sources to include right paths.
    // Somehow the build script complains if they don't include relative paths but just header names.
    // !! Only for local development.
    // transform_pd_headers(&libpd_wrapper_dir);

    // Currently we're not compiling with multi instance support.
    let pd_multi = "false";
    let pd_multi_flag = false;

    // Get info about the target.
    let target_info = get_target_info();

    #[cfg(target_os = "windows")]
    {
        // For windows we need to link pthread.
        // There are some prebuilt libraries for msvc and mingw for architectures x64 and arm64.
        // Mingw support is not tested yet but should work.
        let pthread_root = project_dir.join("pthreads");

        let (pthread_lib_root, pthread_lib_path, pthread_lib_name, pthread_include): (
            PathBuf,
            PathBuf,
            &str,
            PathBuf,
        ) = match &*target_info.arch {
            "x86_64" => match &*(target_info.compiler.unwrap()) {
                "msvc" => {
                    let lib_root = pthread_root
                        .join("msvc")
                        .join("pthreads_x64-windows-static")
                        .join("lib");
                    (
                        lib_root.clone(),
                        lib_root.join("pthreadVC3.lib"),
                        "pthreadVC3",
                        pthread_root
                            .join("msvc")
                            .join("pthreads_x64-windows-static")
                            .join("include"),
                    )
                }
                "gnu" => {
                    let lib_root = pthread_root.join("gnu/x64/lib");
                    (
                        lib_root.clone(),
                        lib_root.join("libpthreadGC2.a"),
                        // Re-visit this
                        "pthreadGC2",
                        pthread_root.join("gnu").join("include"),
                    )
                }
                _ => panic!("Unsupported compiler"),
            },
            "aarch64" => match &*(target_info.compiler.unwrap()) {
                "msvc" => {
                    let lib_root = pthread_root
                        .join("msvc")
                        .join("pthreads_arm64-windows-static")
                        .join("lib");
                    (
                        lib_root.clone(),
                        lib_root.join("pthreadVC3.lib"),
                        "pthreadVC3",
                        pthread_root
                            .join("msvc")
                            .join("pthreads_arm64-windows-static")
                            .join("include"),
                    )
                }
                "gnu" => {
                    let lib_root = pthread_root.join("gnu/aarch64/lib");
                    (
                        lib_root.clone(),
                        lib_root.join("libpthreadGC2.a"),
                        // Re-visit this
                        "pthreadGC2",
                        pthread_root.join("gnu").join("include"),
                    )
                }
                _ => panic!("Unsupported compiler"),
            },
            _ => panic!("Unsupported architecture: {}", target_info.arch),
        };

        let lib_destination = Config::new("libpd")
            .define("PD_EXTRA", PD_EXTRA)
            .define("PD_LOCALE", PD_LOCALE)
            .define("PD_MULTI", pd_multi)
            .define("PD_UTILS", PD_UTILS)
            .cflag(format!("-I{}", pd_source.to_str().unwrap()))
            .cflag(format!("-I{}", libpd_wrapper_dir.to_str().unwrap()))
            .cflag(format!("-I{}", libpd_wrapper_util_dir.to_str().unwrap()))
            .cflag(format!("-DPD_FLOATSIZE={PD_FLOATSIZE}"))
            .define("CMAKE_THREAD_LIBS_INIT", pthread_lib_path.to_str().unwrap())
            .define("PTHREADS_INCLUDE_DIR", pthread_include.to_str().unwrap())
            .no_build_target(true)
            .always_configure(true)
            .very_verbose(true)
            .build();

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
        if !pd_multi_flag {
            println!("cargo:rustc-link-lib=static=libpd-static");
        } else {
            println!("cargo:rustc-link-lib=static=libpd-multi-static");
        }
    }

    #[cfg(target_os = "linux")]
    {
        // I love linux.. everything is concise and simple :)

        let lib_destination = Config::new("libpd")
            .define("PD_EXTRA", PD_EXTRA)
            .define("PD_LOCALE", PD_LOCALE)
            .define("PD_MULTI", pd_multi)
            .define("PD_UTILS", PD_UTILS)
            .cflag(format!("-I{}", pd_source.to_str().unwrap()))
            .cflag(format!("-I{}", libpd_wrapper_dir.to_str().unwrap()))
            .cflag(format!("-I{}", libpd_wrapper_util_dir.to_str().unwrap()))
            .cflag(format!("-DPD_FLOATSIZE={PD_FLOATSIZE}"))
            .no_build_target(true)
            .always_configure(true)
            .very_verbose(true)
            .build();

        let library_root = lib_destination.join("build/libs");
        println!("cargo:rustc-link-search={}", library_root.to_string_lossy());

        if !pd_multi_flag {
            println!("cargo:rustc-link-lib=static=pd");
        } else {
            println!("cargo:rustc-link-lib=static=pd-multi");
        }
    }

    #[cfg(target_os = "macos")]
    {
        let lib_destination = Config::new("libpd")
            .define("PD_EXTRA", PD_EXTRA)
            .define("PD_LOCALE", PD_LOCALE)
            .define("PD_MULTI", pd_multi)
            .define("PD_UTILS", PD_UTILS)
            .cflag(format!("-I{}", pd_source.to_str().unwrap()))
            .cflag(format!("-I{}", libpd_wrapper_dir.to_str().unwrap()))
            .cflag(format!("-I{}", libpd_wrapper_util_dir.to_str().unwrap()))
            .cflag(format!("-DPD_FLOATSIZE={PD_FLOATSIZE}"))
            .define("CMAKE_OSX_ARCHITECTURES", "x86_64;arm64")
            .no_build_target(true)
            .always_configure(true)
            .very_verbose(true)
            .build();

        let library_root = lib_destination.join("build/libs");

        // Look for libpd
        println!("cargo:rustc-link-search={}", library_root.to_string_lossy());

        // Link libpd
        if !pd_multi_flag {
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
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .rustfmt_bindings(true)
        .clang_arg(format!("-I{}", pd_source.to_str().unwrap()))
        .clang_arg(format!("-I{}", libpd_wrapper_dir.to_str().unwrap()))
        .clang_arg(format!("-I{}", libpd_wrapper_util_dir.to_str().unwrap()))
        // This is important to generate the right types.
        .clang_arg(format!("-DPD_FLOATSIZE={PD_FLOATSIZE}"))
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

/// This is needed because somehow headers can not be found even if they are in the include path.
///
/// This is why we transform them to relative paths.
fn transform_pd_headers(base: &Path) {
    let libpd_wrapper_util_dir = base.join("util");
    let z_print_util_h = libpd_wrapper_util_dir.join("z_print_util.h");
    let z_queued_h = libpd_wrapper_util_dir.join("z_queued.h");
    let x_libpdreceive_h = base.join("x_libpdreceive.h");
    let z_libpd_h = base.join("z_libpd.h");

    // z_libpd.h
    let data = std::fs::read_to_string(&z_libpd_h).expect("Unable to read file");
    let data = data.replace(
        "#include \"m_pd.h\"",
        "#include \"../pure-data/src/m_pd.h\"",
    );
    std::fs::write(&z_libpd_h, data).expect("Unable to write file");

    // x_libpdreceive.h
    let data = std::fs::read_to_string(&x_libpdreceive_h).expect("Unable to read file");
    let data = data.replace(
        "#include \"m_pd.h\"",
        "#include \"../pure-data/src/m_pd.h\"",
    );
    std::fs::write(&x_libpdreceive_h, data).expect("Unable to write file");

    // z_queued.h
    let data = std::fs::read_to_string(&z_queued_h).expect("Unable to read file");
    let data = data.replace("#include \"z_libpd.h\"", "#include \"../z_libpd.h\"");
    std::fs::write(&z_queued_h, data).expect("Unable to write file");

    // z_print_util.h
    let data = std::fs::read_to_string(&z_print_util_h).expect("Unable to read file");
    let data = data.replace("#include \"z_libpd.h\"", "#include \"../z_libpd.h\"");
    std::fs::write(&z_print_util_h, data).expect("Unable to write file");
}
