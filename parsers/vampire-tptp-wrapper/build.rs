use std::env;
use std::path::PathBuf;

fn main() {
    let vampire_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("vampire");

    // Step 1: Build vampire_lib via CMake (same approach as vampire-sys build.rs)
    let dst = cmake::Config::new(&vampire_path)
        .define("CMAKE_BUILD_TYPE", "Release")
        .build_target("vampire_lib")
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=vampire_lib");

    // Step 2: Compile the C++ bridge with matching flags
    cc::Build::new()
        .cpp(true)
        .file("cpp/parse_bridge.cpp")
        .std("c++17")
        .flag("-fno-rtti")
        .flag("-fno-threadsafe-statics")
        .include(&vampire_path)
        .include(vampire_path.join("viras/src"))
        .include(vampire_path.join("mini-gmp-6.3.0"))
        .include(vampire_path.join("cadical/src"))
        .define("VDEBUG", "0")
        .define("CHECK_LEAKS", "0")
        .define("VTIME_PROFILING", "0")
        .define("VZ3", "0")
        .define("NBUILD", "1")
        .compile("parse_bridge");

    // Step 3: Link C++ standard library
    let target = env::var("TARGET").unwrap();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
    }

    // Link pthreads (required by Vampire)
    println!("cargo:rustc-link-lib=pthread");

    // Rerun if bridge changes
    println!("cargo:rerun-if-changed=cpp/parse_bridge.cpp");
}
