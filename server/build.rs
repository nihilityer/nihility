use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // 获取构建配置（debug 或 release）
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // 仅在 release 模式下构建前端
    if profile == "release" {
        println!("cargo:warning=Building frontend for release...");

        // 获取项目根目录（server的父目录）
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let frontend_dir = manifest_dir.parent().unwrap().join("frontend");

        // 检查 frontend 目录是否存在
        if !frontend_dir.exists() {
            panic!(
                "Frontend directory not found at: {}",
                frontend_dir.display()
            );
        }

        println!(
            "cargo:warning=Frontend directory: {}",
            frontend_dir.display()
        );

        // 检查 package.json 是否存在
        let package_json = frontend_dir.join("package.json");
        if !package_json.exists() {
            panic!("package.json not found at: {}", package_json.display());
        }

        // 执行 npm install（确保依赖已安装）
        println!("cargo:warning=Running npm install...");
        let npm_install = Command::new("npm")
            .arg("install")
            .current_dir(&frontend_dir)
            .status();

        match npm_install {
            Ok(status) if status.success() => {
                println!("cargo:warning=npm install completed successfully");
            }
            Ok(status) => {
                panic!("npm install failed with status: {}", status);
            }
            Err(e) => {
                panic!("Failed to execute npm install: {}", e);
            }
        }

        // 执行 npm run build
        println!("cargo:warning=Running npm run build...");
        let npm_build = Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(&frontend_dir)
            .status();

        match npm_build {
            Ok(status) if status.success() => {
                println!("cargo:warning=Frontend build completed successfully");
            }
            Ok(status) => {
                panic!("npm run build failed with status: {}", status);
            }
            Err(e) => {
                panic!("Failed to execute npm run build: {}", e);
            }
        }

        // 检查构建产物是否存在
        let dist_dir = frontend_dir.join("dist");
        if !dist_dir.exists() {
            panic!("Frontend build output not found at: {}", dist_dir.display());
        }

        println!(
            "cargo:warning=Frontend build output verified at: {}",
            dist_dir.display()
        );
    } else {
        println!("cargo:warning=Skipping frontend build in {} mode", profile);
        println!("cargo:warning=Frontend assets will be embedded from existing dist/ directory");
    }

    // 告诉 Cargo 当这些文件变化时重新运行 build script
    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/package-lock.json");
    println!("cargo:rerun-if-changed=../frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=../frontend/tsconfig.json");

    // 监控 frontend/src 目录的变化
    if let Ok(entries) = std::fs::read_dir("../frontend/src") {
        for entry in entries.flatten() {
            if let Ok(path) = entry.path().canonicalize() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
