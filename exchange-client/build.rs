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

    // 打包 Python 脚本 - 只监控关键源代码文件
    println!("cargo:rerun-if-changed=src/metatrader5/script/main.py");
    println!("cargo:rerun-if-changed=src/metatrader5/script/pyproject.toml");
    println!("cargo:rerun-if-changed=src/metatrader5/script/api");
    println!("cargo:rerun-if-changed=src/metatrader5/script/mt5_terminal");
    println!("cargo:rerun-if-changed=src/metatrader5/script/parse.py");
    println!("cargo:rerun-if-changed=src/metatrader5/script/util.py");
    println!("cargo:rerun-if-changed=src/metatrader5/script/start_terminals.py");
    println!("cargo:rerun-if-changed=src/metatrader5/script/mt5_terminal.py");
    println!("cargo:rerun-if-changed=src/metatrader5/script/terminals_config.json");

    #[cfg(target_os = "windows")]
    {
        // 设置script目录路径
        let script_dir = Path::new(&project_root).join("src").join("metatrader5").join("script");
        
        // 打包前清理可能影响构建检测的临时文件和目录
        let pycache_dir = script_dir.join("__pycache__");
        let build_dir = script_dir.join("build");
        let dist_dir = script_dir.join("dist");
        
        if pycache_dir.exists() {
            let _ = fs::remove_dir_all(&pycache_dir);
        }
        if build_dir.exists() {
            let _ = fs::remove_dir_all(&build_dir);
        }
        if dist_dir.exists() {
            let _ = fs::remove_dir_all(&dist_dir);
        }
        
        // 使用uv运行pyinstaller进行打包
        // 完整命令: uv run pyinstaller -c -F --clean --name MetaTrader5-x86_64-pc-windows-msvc --distpath <target_dir> main.py
        let status = Command::new("uv")
            .args(&[
                "run",
                "pyinstaller",
                "-c",
                "-F",
                "--clean",
                "--name",
                "MetaTrader5-x86_64-pc-windows-msvc",
                "--distpath",
                target_dir.to_str().unwrap(),
                "main.py"
            ])
            .current_dir(&script_dir)  // 设置工作目录为script目录
            .status()
            .expect("Failed to execute uv run pyinstaller");

        if status.success() {
            println!("打包成功");
            // 清理pyinstaller生成的临时文件
            let spec_file = script_dir.join("MetaTrader5-x86_64-pc-windows-msvc.spec");
            let build_dir = script_dir.join("build");
            let dist_dir = script_dir.join("dist");
            
            if spec_file.exists() {
                let _ = fs::remove_file(spec_file);
            }
            if build_dir.exists() {
                let _ = fs::remove_dir_all(build_dir);
            }
            if dist_dir.exists() {
                let _ = fs::remove_dir_all(dist_dir);
            }
        } else {
            panic!("Failed to build Python executable with uv");
        }
    }

    
    Ok(())
}