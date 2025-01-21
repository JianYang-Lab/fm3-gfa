use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    // using cargo output dir as the build dir
    let out_dir = env::var("OUT_DIR").unwrap();
    // build cc
    let mut build = cc::Build::new();

    // Basic configuration
    build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-fPIC")
        .warnings(false)
        .includes(&["ogdf"]);

    // Only support macOS and Linux
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "macos" {
        // macOS
        build
            .flag("-w")
            .flag("-stdlib=libc++")
            .flag("-Wno-unused-parameter")
            .flag("-Wno-unused-variable");
    } else if env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
        // Linux
        build
            .flag("-w")
            .flag("-Wno-class-memaccess")
            .flag("-Wno-unused-parameter")
            .flag("-Wno-unused-variable")
            .flag("-Wno-deprecated-declarations");
    }

    // Add all cpp files in the ogdf directory
    for entry in WalkDir::new("ogdf").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "cpp" {
                println!("cargo:warning=Adding source file: {}", path.display());
                build.file(path);
            }
        }
    }

    // Add wrapper file
    build.file("src/wrapper.cpp");

    // Just compile it!!!
    build.compile("ogdf_wrapper");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_arg("-xc++")
        .clang_arg("-std=c++17")
        .clang_arg("-I./ogdf") // include ogdf headers
        .allowlist_function("init_layout")
        .allowlist_function("run_layout")
        .allowlist_function("destroy_layout")
        .allowlist_function("free_string")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link the C++ standard library
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if env::var("CARGO_CFG_TARGET_OS").unwrap() == "macos" {
        println!("cargo:rustc-link-lib=dylib=c++");
    }

    // When the wrapper and ogdf files change, rerun the build script
    println!("cargo:rerun-if-changed=ogdf");
    println!("cargo:rerun-if-changed=src/wrapper.cpp");
    println!("cargo:rerun-if-changed=src/wrapper.h");
}
