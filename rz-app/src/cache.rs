//! 最后一次有效读数的本地缓存。
//!
//! 无线鼠标休眠 / 未插上时，界面不至于退化成一片 `0`，
//! 而是显示上次成功读到的真实配置（并标注「休眠中」）。

use rz_core::device::Caps;
use rz_core::{Battery, DeviceState, DpiProfile, DpiStage};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn path() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("RazerControl").join("last_state.txt")
}

pub fn save(st: &DeviceState) {
    if st.profile.stages.is_empty() && !st.caps.battery {
        return;
    }
    let stages: Vec<String> = st.profile.stages.iter().map(|s| s.x.to_string()).collect();
    let caps = format!(
        "{}{}{}{}{}{}{}{}{}{}",
        b(st.caps.dpi),
        b(st.caps.dpi_stages),
        b(st.caps.poll_rate),
        b(st.caps.battery),
        b(st.caps.charging),
        b(st.caps.sleep),
        b(st.caps.low_battery),
        b(st.caps.lighting),
        b(st.caps.wireless),
        b(st.caps.calibration)
    );
    let body = format!(
        "name={}\npid={}\nfirmware={}\nserial={}\nwireless={}\ndpi={},{}\nactive={}\nstages={}\npoll_hz={}\npoll_code={}\nsleep={}\nlowbatt={}\nbattery={}\ncharging={}\nmax_dpi={}\nmax_poll={}\nhas_rgb={}\ncaps={}\nbrightness={}\nliftoff={}\ntracking={}\nedition={}\n",
        st.name,
        st.pid,
        st.firmware,
        st.serial,
        b(st.wireless),
        st.dpi.0,
        st.dpi.1,
        st.profile.active,
        stages.join(","),
        st.poll_hz,
        st.poll_code,
        st.sleep_secs,
        st.low_battery_percent,
        st.battery.percent,
        b(st.battery.charging),
        st.max_dpi,
        st.max_poll,
        b(st.has_rgb),
        caps,
        st.brightness.unwrap_or(0),
        st.liftoff,
        st.tracking,
        st.edition,
    );
    let p = path();
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(p, body);
}

fn b(v: bool) -> u8 {
    if v {
        1
    } else {
        0
    }
}

pub fn load() -> Option<DeviceState> {
    let text = fs::read_to_string(path()).ok()?;
    let mut m: HashMap<&str, &str> = HashMap::new();
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            m.insert(k.trim(), v.trim());
        }
    }
    let num = |k: &str| -> Option<u32> { m.get(k).and_then(|v| v.parse().ok()) };
    let flag = |k: &str| -> bool { m.get(k).map(|v| *v == "1").unwrap_or(false) };

    let stages: Vec<DpiStage> = m
        .get("stages")
        .map(|s| {
            s.split(',')
                .filter_map(|x| x.trim().parse::<u16>().ok())
                .map(|x| DpiStage { x, y: x })
                .collect()
        })
        .unwrap_or_default();
    if stages.is_empty() {
        return None;
    }
    let caps_s = m.get("caps").copied().unwrap_or("0000000000");
    let cv: Vec<bool> = caps_s.chars().map(|c| c == '1').collect();
    let get = |i: usize| cv.get(i).copied().unwrap_or(false);

    Some(DeviceState {
        name: m.get("name").copied().unwrap_or("Razer 鼠标").to_string(),
        edition: num("edition").unwrap_or(0) as u8,
        pid: num("pid").unwrap_or(0) as u16,
        firmware: m.get("firmware").copied().unwrap_or("").to_string(),
        serial: m.get("serial").copied().unwrap_or("").to_string(),
        wireless: flag("wireless"),
        dpi: (
            m.get("dpi")
                .and_then(|v| v.split(',').next())
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            m.get("dpi")
                .and_then(|v| v.split(',').nth(1))
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
        ),
        profile: DpiProfile {
            active: num("active").unwrap_or(1) as u8,
            stages,
        },
        poll_hz: num("poll_hz").unwrap_or(0) as u16,
        poll_code: num("poll_code").unwrap_or(0) as u8,
        sleep_secs: num("sleep").unwrap_or(0) as u16,
        low_battery_percent: num("lowbatt").unwrap_or(0) as u8,
        battery: Battery {
            percent: num("battery").unwrap_or(0) as u8,
            charging: flag("charging"),
            present: true,
        },
        brightness: num("brightness").map(|v| v as u8),
        liftoff: num("liftoff").unwrap_or(2) as u8,
        tracking: num("tracking").unwrap_or(1) as u8,
        caps: Caps {
            dpi: get(0),
            dpi_stages: get(1),
            poll_rate: get(2),
            battery: get(3),
            charging: get(4),
            sleep: get(5),
            low_battery: get(6),
            lighting: get(7),
            wireless: get(8),
            calibration: get(9),
        },
        max_dpi: num("max_dpi").unwrap_or(16000) as u16,
        max_poll: num("max_poll").unwrap_or(1000) as u16,
        has_rgb: flag("has_rgb"),
    })
}
