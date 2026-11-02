//! 用 windres 把 assets/app.rc（多尺寸图标组）编译成对象文件并链接进 exe。
//! 这样任务栏、资源管理器、Alt-Tab 都能取到原生尺寸的图标，不会互相拉伸。

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/razer.ico");

    let out_dir = match std::env::var("OUT_DIR") {
        Ok(v) => v,
        Err(_) => return,
    };
    let obj = format!("{}/app_res.o", out_dir);
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());

    // 常见工具链里的资源编译器
    let candidates = [
        "windres".to_string(),
        format!("{}/../../mingw64/bin/windres.exe", manifest),
        "x86_64-w64-mingw32-windres".to_string(),
        "llvm-rc".to_string(),
    ];
    for exe in candidates.iter() {
        let ok = Command::new(exe)
            .current_dir(&manifest)
            .args(["assets/app.rc", "-O", "coff", "-o", &obj])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            println!("cargo:rustc-link-arg={}", obj);
            return;
        }
    }
    // 没有资源编译器就跳过（图标仍由 winit 在运行时设置）
    println!("cargo:warning=windres 不可用，跳过 exe 图标资源");
}
