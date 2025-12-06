use std::{env, path::PathBuf};

#[cfg(target_os = "windows")]
fn build_ta_lib_windows() {
    println!("=== Build Script Starting ===");

    // Set library search path
    // let lib_path = PathBuf::from("./src-tauri/libs/ta-lib/lib");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Unable to get CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    let lib_path = manifest_dir
        // .join("src")
        // .join("indicator_engine")
        // .join("libs")
        .join("talib_c")
        .join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // Link TA-Lib
    println!("cargo:rustc-link-lib=static=ta-lib-static");

    // Set header file path
    // let include_path = PathBuf::from("./src-tauri/libs/ta-lib/include");
    let include_path = manifest_dir
        // .join("src")
        // .join("indicator_engine")
        // .join("libs")
        .join("talib_c")
        .join("include");
    println!("Header file path: {}", include_path.display());

    // Set rerun conditions
    println!("cargo:rerun-if-changed=talib_c/include");
    println!("cargo:rerun-if-changed=build.rs");

    // Generate Rust bindings
    let bindings = bindgen::Builder::default()
        // Main header file
        .header(include_path.join("ta_libc.h").to_str().unwrap())
        // Include path
        .clang_arg(format!("-I{}", include_path.display()))
        // Generate full documentation comments
        .generate_comments(true)
        // Use core instead of std to support no_std
        .use_core()
        // Allow generating unsafe code
        .trust_clang_mangling(false)
        // Derive default
        .derive_default(true)
        // Generate bindings
        .generate()
        .expect("Unable to generate bindings");

    // Write generated bindings
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("Output path: {}", out_path.display());

    let binding_path = out_path.join("bindings.rs");
    bindings.write_to_file(&binding_path).expect("Unable to write bindings");

    println!("Binding file generated at: {}", binding_path.display());
    println!("=== Build Script Ending ===");
}

fn build_ta_lib_macos() {
    println!("=== Build Script Starting (macOS) ===");

    // Set library search path
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Unable to get CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    let lib_path = manifest_dir
        // .join("src")
        // .join("indicator_engine")
        // .join("libs")
        .join("talib_c")
        .join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // Link TA-Lib (macOS uses libta-lib.a)
    println!("cargo:rustc-link-lib=static=ta-lib");

    // Set header file path
    let include_path = manifest_dir
        // .join("src")
        // .join("indicator_engine")
        // .join("libs")
        .join("talib_c")
        .join("include");
    println!("Header file path: {}", include_path.display());

    // Set rerun conditions
    println!("cargo:rerun-if-changed=talib_c/include");
    println!("cargo:rerun-if-changed=build.rs");

    // Generate Rust bindings - macOS specific configuration
    let bindings = bindgen::Builder::default()
        // Main header file
        .header(include_path.join("ta_libc.h").to_str().unwrap())
        // Include path
        .clang_arg(format!("-I{}", include_path.display()))
        // Generate full documentation comments
        .generate_comments(true)
        // Use core instead of std to support no_std
        .use_core()
        // Allow generating unsafe code
        .trust_clang_mangling(false)
        // Derive default
        .derive_default(true)
        // macOS specific: Ignore platform-specific structs to avoid size mismatch
        .blocklist_type("_Mbstatet")
        .blocklist_type("mbstate_t")
        // Force all enums to use c_int type
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        // Add clang arguments to force enums as signed types
        .clang_arg("-fno-unsigned-char")
        .clang_arg("-fsigned-char")
        // Add macOS specific clang arguments
        .clang_arg("-target")
        .clang_arg("x86_64-apple-darwin")
        // Generate bindings
        .generate()
        .expect("Unable to generate bindings");

    // Write generated bindings
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("Output path: {}", out_path.display());

    let binding_path = out_path.join("bindings.rs");
    bindings.write_to_file(&binding_path).expect("Unable to write bindings");

    println!("Binding file generated at: {}", binding_path.display());
    println!("=== Build Script Ending (macOS) ===");
}

fn main() {
    // Build TA-Lib on Windows
    #[cfg(target_os = "windows")]
    {
        build_ta_lib_windows();
    }

    // Build TA-Lib on macOS
    #[cfg(target_os = "macos")]
    {
        build_ta_lib_macos();
    }

    // Skip build on other platforms
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        println!("cargo:warning=Skipping TA-Lib build on unsupported platform");
    }
}
