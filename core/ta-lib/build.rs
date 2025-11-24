use std::{env, path::PathBuf};

#[cfg(target_os = "windows")]
fn build_ta_lib_windows() {
    println!("=== Build Script Starting ===");

    // 设置库的搜索路径
    // let lib_path = PathBuf::from("./src-tauri/libs/ta-lib/lib");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("无法获取 CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    let lib_path = manifest_dir
        // .join("src")
        // .join("indicator_engine")
        // .join("libs")
        .join("talib_c")
        .join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // 链接 TA-Lib
    println!("cargo:rustc-link-lib=static=ta-lib-static");

    // 设置头文件路径
    // let include_path = PathBuf::from("./src-tauri/libs/ta-lib/include");
    let include_path = manifest_dir
        // .join("src")
        // .join("indicator_engine")
        // .join("libs")
        .join("talib_c")
        .join("include");
    println!("头文件路径: {}", include_path.display());

    // // 设置重新运行的条件
    println!("cargo:rerun-if-changed=talib_c/include");
    println!("cargo:rerun-if-changed=build.rs");

    // // 生成 Rust 绑定
    let bindings = bindgen::Builder::default()
        //     // 主头文件
        .header(include_path.join("ta_libc.h").to_str().unwrap())
        //     // 包含路径
        .clang_arg(format!("-I{}", include_path.display()))
        //     // 生成完整的文档注释
        .generate_comments(true)
        //     // 使用 core 而不是 std，以支持 no_std
        .use_core()
        //     // 允许生成不安全的代码
        .trust_clang_mangling(false)
        //     // 生成块大小
        .derive_default(true)
        //     // 生成绑定
        .generate()
        .expect("无法生成绑定");

    // // 写入生成的绑定
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("输出路径: {}", out_path.display());

    let binding_path = out_path.join("bindings.rs");
    bindings.write_to_file(&binding_path).expect("无法写入绑定");

    println!("绑定文件已生成在: {}", binding_path.display());
    println!("=== Build Script Ending ===");
}

fn build_ta_lib_macos() {
    println!("=== Build Script Starting (macOS) ===");

    // 设置库的搜索路径
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("无法获取 CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    let lib_path = manifest_dir
        // .join("src")
        // .join("indicator_engine")
        // .join("libs")
        .join("talib_c")
        .join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // 链接 TA-Lib (macOS 使用 libta-lib.a)
    println!("cargo:rustc-link-lib=static=ta-lib");

    // 设置头文件路径
    let include_path = manifest_dir
        // .join("src")
        // .join("indicator_engine")
        // .join("libs")
        .join("talib_c")
        .join("include");
    println!("头文件路径: {}", include_path.display());

    // 设置重新运行的条件
    println!("cargo:rerun-if-changed=talib_c/include");
    println!("cargo:rerun-if-changed=build.rs");

    // 生成 Rust 绑定 - macOS 特定配置
    let bindings = bindgen::Builder::default()
        // 主头文件
        .header(include_path.join("ta_libc.h").to_str().unwrap())
        // 包含路径
        .clang_arg(format!("-I{}", include_path.display()))
        // 生成完整的文档注释
        .generate_comments(true)
        // 使用 core 而不是 std，以支持 no_std
        .use_core()
        // 允许生成不安全的代码
        .trust_clang_mangling(false)
        // 生成块大小
        .derive_default(true)
        // macOS 特定：忽略平台相关的结构体以避免大小不匹配
        .blocklist_type("_Mbstatet")
        .blocklist_type("mbstate_t")
        // 强制所有枚举使用 c_int 类型
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        // 添加 clang 参数强制枚举为有符号类型
        .clang_arg("-fno-unsigned-char")
        .clang_arg("-fsigned-char")
        // 添加 macOS 特定的 clang 参数
        .clang_arg("-target")
        .clang_arg("x86_64-apple-darwin")
        // 生成绑定
        .generate()
        .expect("无法生成绑定");

    // 写入生成的绑定
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("输出路径: {}", out_path.display());

    let binding_path = out_path.join("bindings.rs");
    bindings.write_to_file(&binding_path).expect("无法写入绑定");

    println!("绑定文件已生成在: {}", binding_path.display());
    println!("=== Build Script Ending (macOS) ===");
}

fn main() {
    // 在 Windows 环境下构建 TA-Lib
    #[cfg(target_os = "windows")]
    {
        build_ta_lib_windows();
    }

    // 在 macOS 环境下构建 TA-Lib
    #[cfg(target_os = "macos")]
    {
        build_ta_lib_macos();
    }

    // 在其他环境下跳过构建
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        println!("cargo:warning=Skipping TA-Lib build on unsupported platform");
    }
}
