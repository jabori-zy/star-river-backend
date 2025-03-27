use std::process::Command;
use std::path::Path;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // 设置资源目录
    // let resources_dir = Path::new(&project_root).join("src").join("metatrader5").join("script");
    let target_dir = Path::new(&project_root).join("src").join("metatrader5").join("bin").join(match target_os.as_str() {
        "windows" => "windows",
        "macos" => "macos",
        _ => "linux",
    });

    // 创建目录
    fs::create_dir_all(&target_dir).unwrap();

    // 打包 Python 脚本
    println!("cargo:rerun-if-changed=src/metatrader5/script");

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
                "src/metatrader5/script/main.py"
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

    
    Ok(())
}
