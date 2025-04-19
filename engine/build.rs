use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::path::Path;
use std::fs;

fn build_ta_lib() {
    println!("=== Build Script Starting ===");

    // 设置库的搜索路径
    // let lib_path = PathBuf::from("./src-tauri/libs/ta-lib/lib");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("无法获取 CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    let lib_path = manifest_dir.join("src").join("indicator_engine").join("libs").join("ta-lib").join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // 链接 TA-Lib
    println!("cargo:rustc-link-lib=static=ta-lib-static");

    // 设置头文件路径
    // let include_path = PathBuf::from("./src-tauri/libs/ta-lib/include");
    let include_path = manifest_dir.join("src").join("indicator_engine").join("libs").join("ta-lib").join("include");
    println!("头文件路径: {}", include_path.display());

    // // 设置重新运行的条件
    println!("cargo:rerun-if-changed=libs/ta-lib/include");
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

fn build_mt5_client() {
    println!("=== Build Script Starting ===");
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // 设置资源目录
    // let resources_dir = Path::new(&project_root).join("src").join("metatrader5").join("script");
    let target_dir = Path::new(&project_root).join("src").join("exchange_engine").join("bin").join("windows");

    // 创建目录
    fs::create_dir_all(&target_dir).unwrap();

    // 打包 Python 脚本
    println!("cargo:rerun-if-changed=src/exchange_engine/script");

    #[cfg(target_os = "windows")]
    {
        // Windows 平台打包
        let status = Command::new("pyinstaller")
            .args(&[
                "-c",
                "-F",
                "--clean",
                "--name",
                "MetaTrader5-x86_64-pc-windows-msvc",
                "--distpath",
                target_dir.to_str().unwrap(),
                "src/exchange_engine/script/main.py"
            ])
            .status()
            .expect("Failed to execute pyinstaller");

        if status.success() {
            println!("打包成功");
            // 清理.spec文件
            let spec_file = Path::new(&project_root).join("MetaTrader5-x86_64-pc-windows-msvc.spec");
            if spec_file.exists() {
                fs::remove_file(spec_file).unwrap();
            }
        }

        if !status.success() {
            panic!("Failed to build Python executable");
        }
    }

    
    println!("=== Build Script Ending ===");
}

fn main() {
    // 首先告诉Cargo不要自动检测文件变化
    // println!("cargo:rerun-if-changed=.");
    // // 然后明确指定需要监控的文件
    // println!("cargo:rerun-if-changed=src/exchange_engine/script");
    // println!("cargo:rerun-if-changed=libs/ta-lib/include");
    // println!("cargo:rerun-if-changed=build.rs");
    
    build_ta_lib();
    // build_mt5_client();
}
