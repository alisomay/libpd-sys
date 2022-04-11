use cmake::Config;
use std::process::Command;
use std::{
    env,
    path::{Path, PathBuf},
};

// All compilation settings defined in libpd documentation.
// Maybe include some more of these later?

// UTIL=true: compile utilities in libpd_wrapper/util (default)
// EXTRA=true: compile pure-data/extra externals which are then inited in libpd_init() (default)
// MULTI=true: compile with multiple instance support
// LOCALE=false: do not set the LC_NUMERIC number format to the default "C" locale* (default)

// DEBUG=true: compile with debug symbols & no optimizations
// STATIC=true: compile static library (in addition to shared library)
// FAT_LIB=true: compile universal "fat" lib with multiple architectures (macOS only)

// PORTAUDIO=true: compile with portaudio support (currently JAVA jni only)
// JAVA_HOME=/path/to/jdk: specify the path to the Java Development Kit

// TODO: Make pd compilation settings configurable from cargo.
const PD_EXTRA: &str = "true";
const PD_LOCALE: &str = "false";
const PD_UTILS: &str = "true";
const PD_FLOATSIZE: &str = "64";

fn main() {
    let mut project_dir = std::path::PathBuf::new();
    project_dir.push(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    let libpd_dir = project_dir.join("libpd");
    // let pd_source_dir = libpd_dir.join("pure-data/src");
    let libpd_wrapper_dir = libpd_dir.join("libpd_wrapper");

    transform_pd_headers(&libpd_wrapper_dir);

    // Currently we're not compiling with multi instance support.
    let pd_multi = "false";
    let pd_multi_flag = false;

    // Get info about the target.
    let target_info = get_target_info();

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
        "aarch64" => panic!("Windows aarch64 build is waiting for your support!"),
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
        .define("PD_FLOATSIZE", PD_FLOATSIZE)
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
        .define("PD_FLOATSIZE", PD_FLOATSIZE)
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
        .define("PD_FLOATSIZE", PD_FLOATSIZE)
        .define("CMAKE_OSX_ARCHITECTURES", "x86_64;arm64")
        .no_build_target(true)
        .always_configure(true)
        .very_verbose(true)
        .build();

    let library_root = format!("{}/build/libs", lib_destination.as_path().display());
    println!("cargo:rustc-link-search={library_root}");

    #[cfg(target_os = "macos")]
    if !pd_multi_flag {
        thin_fat_lib(&library_root, false);
        match &*target_info.arch {
            // We now have two thin libs, one for each architecture, we need to link the appropriate one.
            // libpd-x86_64.a and libpd-aarch64.a
            "x86_64" => println!("cargo:rustc-link-lib=static=pd-x86_64"),
            "aarch64" => println!("cargo:rustc-link-lib=static=pd-aarch64"),
            _ => panic!("Unsupported architecture"),
        }
    } else {
        thin_fat_lib(&library_root, true);
        // TODO: Test this.
        match &*target_info.arch {
            // We now have two thin libs, one for each architecture, we need to link the appropriate one.
            // libpd-x86_64.a and libpd-aarch64.a
            "x86_64" => println!("cargo:rustc-link-lib=static=pd-multi-x86_64"),
            "aarch64" => println!("cargo:rustc-link-lib=static=pd-multi-aarch64"),
            _ => panic!("Unsupported architecture"),
        }
    }

    #[cfg(target_os = "linux")]
    if !pd_multi_flag {
        println!("cargo:rustc-link-lib=static=pd");
    } else {
        println!("cargo:rustc-link-lib=static=pd-multi");
    }

    #[cfg(target_os = "windows")]
    if !pd_multi_flag {
        println!("cargo:rustc-link-lib=static=pd-static");
    } else {
        println!("cargo:rustc-link-lib=static=pd-multi-static");
    }

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .rustfmt_bindings(true)
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
fn thin_fat_lib(library_root: &str, pd_multi: bool) {
    let mut name = String::from("libpd");
    if pd_multi {
        name = format!("{}-multi", name);
    }
    Command::new("lipo")
        .arg(format!("{library_root}/{name}.a"))
        .arg("-thin")
        // Apple calls aarch64, arm64
        .arg("arm64")
        .arg("-output")
        .arg(format!("{library_root}/{name}-aarch64.a"))
        .spawn()
        .expect("lipo command failed to start");

    Command::new("lipo")
        .arg(format!("{library_root}/{name}.a"))
        .arg("-thin")
        .arg("x86_64")
        .arg("-output")
        .arg(format!("{library_root}/{name}-x86_64.a"))
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
