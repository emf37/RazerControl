//! 系统集成：开机自启、打开配置目录。

use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "RazerControl";

pub fn set_autostart(enabled: bool) {
    let exe = std::env::current_exe().unwrap_or_default();
    if enabled {
        let _ = Command::new("reg")
            .args([
                "add",
                RUN_KEY,
                "/v",
                VALUE,
                "/t",
                "REG_SZ",
                "/d",
                &format!("\"{}\"", exe.display()),
                "/f",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    } else {
        let _ = Command::new("reg")
            .args(["delete", RUN_KEY, "/v", VALUE, "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
}

pub fn open_config_dir() {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let dir = std::path::PathBuf::from(base).join("RazerControl");
    let _ = std::fs::create_dir_all(&dir);
    let _ = Command::new("explorer")
        .arg(dir)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}
