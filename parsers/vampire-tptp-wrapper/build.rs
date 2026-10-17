use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vampire_path = manifest_dir.join("vampire");

    // Build the parser-only static library assembled by our wrapper CMake file.
    let dst = cmake::Config::new(&manifest_dir)
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("CMAKE_BUILD_PARALLEL_LEVEL", "1")
        .env("VAMPIRE_BUILD_JOBS", "1")
        .build_target("vampire_lib")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=vampire_lib");

    // Step 2: Compile the C++ bridge with matching flags
    cc::Build::new()
        .cpp(true)
        .file(manifest_dir.join("cpp/parse_bridge.cpp"))
        .std("c++17")
        .flag("-fno-rtti")
        .flag("-fno-threadsafe-statics")
        .include(&vampire_path)
        .include(vampire_path.join("viras/src"))
        .include(vampire_path.join("mini-gmp-6.3.0"))
        .include(vampire_path.join("cadical/src"))
        .include(vampire_path.join("build"))
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
    println!("cargo:rerun-if-changed=CMakeLists.txt");
    println!("cargo:rerun-if-changed=vampire/CMakeLists.txt");
}
