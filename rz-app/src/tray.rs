//! 系统托盘：Shell_NotifyIconW + 隐藏窗口消息循环 + 右键菜单。
//!
//! 图标用 Razer 官方 ICO（`assets/razer.ico`，内含 16→256 共 12 个尺寸），编译期嵌入，
//! 运行时从 ICONDIR 里按尺寸挑一条交给 `CreateIconFromResourceEx`。
//! 关闭按钮不退出程序而是隐藏窗口，只有托盘的「退出」才真正结束进程。

use std::sync::OnceLock;

const TRAY_CALLBACK: u32 = 0x8001;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_RBUTTONUP: u32 = 0x0205;
const NIM_ADD: u32 = 0;
const NIM_MODIFY: u32 = 1;
const NIM_DELETE: u32 = 2;
const NIF_MESSAGE: u32 = 0x01;
const NIF_ICON: u32 = 0x02;
const NIF_TIP: u32 = 0x04;
const NIF_INFO: u32 = 0x10;
const NIIF_INFO: u32 = 0x01;
const MF_STRING: u32 = 0x0000;
const MF_SEPARATOR: u32 = 0x0800;
const TPM_RETURNCMD: u32 = 0x0100;
const TPM_RIGHTBUTTON: u32 = 0x0002;

pub static ICO: &[u8] = include_bytes!("../assets/razer.ico");

#[repr(C)]
struct Guid {
    a: u32,
    b: u16,
    c: u16,
    d: [u8; 8],
}

#[repr(C)]
struct NotifyIconDataW {
    cb_size: u32,
    hwnd: isize,
    uid: u32,
    flags: u32,
    callback_message: u32,
    hicon: isize,
    tip: [u16; 128],
    state: u32,
    state_mask: u32,
    info: [u16; 256],
    timeout_or_version: u32,
    info_title: [u16; 64],
    info_flags: u32,
    guid_item: Guid,
    balloon_icon: isize,
}

#[repr(C)]
struct WndClassExW {
    cb_size: u32,
    style: u32,
    wnd_proc: Option<unsafe extern "system" fn(isize, u32, usize, isize) -> isize>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: isize,
    icon: isize,
    cursor: isize,
    background: isize,
    menu_name: *const u16,
    class_name: *const u16,
    icon_small: isize,
}

#[repr(C)]
struct Msg {
    hwnd: isize,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: [i32; 2],
    private: u32,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(class: *const WndClassExW) -> u16;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: isize,
        menu: isize,
        instance: isize,
        param: *mut core::ffi::c_void,
    ) -> isize;
    fn DefWindowProcW(hwnd: isize, msg: u32, w: usize, l: isize) -> isize;
    fn DestroyWindow(hwnd: isize) -> i32;
    fn GetMessageW(msg: *mut Msg, hwnd: isize, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn RegisterWindowMessageW(name: *const u16) -> u32;
    fn CreatePopupMenu() -> isize;
    fn DestroyMenu(menu: isize) -> i32;
    fn AppendMenuW(menu: isize, flags: u32, id: usize, text: *const u16) -> i32;
    fn TrackPopupMenu(
        menu: isize,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: isize,
        rect: *const core::ffi::c_void,
    ) -> i32;
    fn SetForegroundWindow(hwnd: isize) -> i32;
    fn GetCursorPos(p: *mut Point) -> i32;
    fn PostMessageW(hwnd: isize, msg: u32, w: usize, l: isize) -> i32;
    fn CreateIconFromResourceEx(
        bits: *const u8,
        size: u32,
        icon: i32,
        version: u32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> isize;
    fn Shell_NotifyIconW(op: u32, data: *const NotifyIconDataW) -> i32;
    fn FindWindowW(class: *const u16, name: *const u16) -> isize;
    fn ShowWindow(hwnd: isize, cmd: i32) -> i32;
    fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
    fn CallWindowProcW(prev: isize, hwnd: isize, msg: u32, w: usize, l: isize) -> isize;
    fn GetDpiForSystem() -> u32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> isize;
}

const GWLP_WNDPROC: i32 = -4;
const WM_CLOSE: u32 = 0x0010;

static CTX: OnceLock<egui::Context> = OnceLock::new();
static CLOSE_HOOKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static ALLOW_CLOSE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static ORIG_PROC: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static TASKBAR_CREATED: OnceLock<u32> = OnceLock::new();

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ---------------- ICO ----------------

#[derive(Clone, Copy)]
pub struct IcoEntry {
    pub width: u32,
    pub height: u32,
    pub offset: usize,
    pub size: usize,
}

pub fn ico_entries(data: &[u8]) -> Vec<IcoEntry> {
    let mut out = Vec::new();
    if data.len() < 6 || data[0] != 0 || data[1] != 0 || data[2] != 1 || data[3] != 0 {
        return out;
    }
    let count = u16::from_le_bytes([data[4], data[5]]) as usize;
    for i in 0..count {
        let o = 6 + i * 16;
        if o + 16 > data.len() {
            break;
        }
        let w = if data[o] == 0 { 256 } else { data[o] as u32 };
        let h = if data[o + 1] == 0 { 256 } else { data[o + 1] as u32 };
        let size = u32::from_le_bytes([data[o + 8], data[o + 9], data[o + 10], data[o + 11]]) as usize;
        let off =
            u32::from_le_bytes([data[o + 12], data[o + 13], data[o + 14], data[o + 15]]) as usize;
        if off + size <= data.len() {
            out.push(IcoEntry {
                width: w,
                height: h,
                offset: off,
                size,
            });
        }
    }
    out
}

pub fn ico_pick(data: &[u8], want: u32) -> Option<IcoEntry> {
    let list = ico_entries(data);
    list.iter()
        .filter(|e| e.width >= want && e.height >= want)
        .min_by_key(|e| e.width)
        .or_else(|| list.iter().max_by_key(|e| e.width))
        .copied()
}

fn make_icon() -> isize {
    if let Some(e) = ico_pick(ICO, 16) {
        let slice = &ICO[e.offset..e.offset + e.size];
        let h = unsafe {
            CreateIconFromResourceEx(
                slice.as_ptr(),
                slice.len() as u32,
                1,
                0x0003_0000,
                e.width as i32,
                e.height as i32,
                0,
            )
        };
        if h != 0 {
            return h;
        }
    }
    0
}

/// 解码单个 ICO 条目为 egui 图标数据
fn decode_entry(e: IcoEntry) -> Option<egui::IconData> {
    let slice = &ICO[e.offset..e.offset + e.size];
    // Vista+ 的 ICO 允许条目是 PNG
    if slice.len() > 8 && slice[..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        if let Ok(img) = image::load_from_memory(slice) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            return Some(egui::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            });
        }
    }
    // 32bpp DIB 条目：BITMAPINFOHEADER + 自底向上 BGRA
    if slice.len() > 40 {
        let bi_size = u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]) as usize;
        let bits = u16::from_le_bytes([slice[14], slice[15]]);
        let (w, hh) = (e.width as usize, e.height as usize);
        if bi_size == 40 && bits == 32 && w > 0 && hh > 0 && slice.len() >= 40 + w * hh * 4 {
            let pix = &slice[40..];
            let mut rgba = Vec::with_capacity(w * hh * 4);
            for row in (0..hh).rev() {
                for col in 0..w {
                    let i = (row * w + col) * 4;
                    rgba.push(pix[i + 2]);
                    rgba.push(pix[i + 1]);
                    rgba.push(pix[i]);
                    rgba.push(pix[i + 3]);
                }
            }
            return Some(egui::IconData {
                rgba,
                width: w as u32,
                height: hh as u32,
            });
        }
    }
    None
}

/// 给 eframe 用的窗口图标。
///
/// 任务栏按钮在 100% DPI 下就是 32×32，直接给原生 32 尺寸才是 1:1；
/// 之前把 256×256 丢过去，Windows 会降采样成 32，边缘就发虚。
pub fn window_icon() -> Option<egui::IconData> {
    // 任务栏按钮 = 32 逻辑像素，按系统 DPI 换算成物理像素再去图标组里找原生尺寸
    let dpi = unsafe { GetDpiForSystem() }.max(96) as f32;
    let base = ((32.0 * dpi / 96.0).round() as u32).max(16);
    let entries = ico_entries(ICO);
    let pick = |want: u32| -> Option<IcoEntry> {
        entries
            .iter()
            .find(|e| e.width == want && e.height == want)
            .copied()
            .or_else(|| {
                entries
                    .iter()
                    .filter(|e| e.width >= want && e.height >= want)
                    .min_by_key(|e| e.width)
                    .copied()
            })
    };
    for want in [base, base * 2, 256] {
        if let Some(e) = pick(want) {
            if let Some(d) = decode_entry(e) {
                return Some(d);
            }
        }
    }
    entries.iter().rev().find_map(|e| decode_entry(*e))
}

// ---------------- 主窗口 ----------------

fn main_hwnd() -> isize {
    unsafe { FindWindowW(wide("Window Class").as_ptr(), wide("RazerControl").as_ptr()) }
}

pub fn show_main() {
    let h = main_hwnd();
    if h != 0 {
        unsafe {
            ShowWindow(h, 9); // SW_RESTORE（SW_HIDE 后单次 SW_SHOW 可能无效）
            ShowWindow(h, 5); // SW_SHOW
            SetForegroundWindow(h);
        }
    }
    if let Some(ctx) = CTX.get() {
        ctx.request_repaint();
    }
}

pub fn hide_main() {
    let h = main_hwnd();
    if h != 0 {
        unsafe {
            ShowWindow(h, 0); // SW_HIDE
        }
    }
}

/// 子类化主窗口：把 WM_CLOSE（任务栏右键「关闭窗口」/ Alt+F4）改成「收进托盘」
unsafe extern "system" fn close_hook(hwnd: isize, msg: u32, w: usize, l: isize) -> isize {
    if msg == WM_CLOSE && !ALLOW_CLOSE.load(std::sync::atomic::Ordering::Relaxed) {
        ShowWindow(hwnd, 0); // SW_HIDE
        notify("RazerControl", "已最小化到托盘 · 右键托盘图标可退出");
        return 0;
    }
    let prev = ORIG_PROC.load(std::sync::atomic::Ordering::Relaxed);
    if prev == 0 {
        return DefWindowProcW(hwnd, msg, w, l);
    }
    CallWindowProcW(prev, hwnd, msg, w, l)
}

/// 安装 WM_CLOSE 拦截（窗口创建后调用一次）
pub fn hook_close() {
    if CLOSE_HOOKED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let h = main_hwnd();
    if h == 0 {
        CLOSE_HOOKED.store(false, std::sync::atomic::Ordering::Relaxed);
        return;
    }
    let prev = unsafe { SetWindowLongPtrW(h, GWLP_WNDPROC, close_hook as usize as isize) };
    if prev != 0 {
        ORIG_PROC.store(prev, std::sync::atomic::Ordering::Relaxed);
    } else {
        CLOSE_HOOKED.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

pub fn quit_app() {
    ALLOW_CLOSE.store(true, std::sync::atomic::Ordering::Relaxed);
    let h = main_hwnd();
    if h != 0 {
        unsafe {
            PostMessageW(h, WM_CLOSE, 0, 0); // 走正常关闭流程（winit 结束事件循环）
        }
    }
    if let Some(ctx) = CTX.get() {
        ctx.request_repaint();
    }
    // 兜底：2 秒内没退出就强制结束，避免托盘「退出」失效
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(2));
        std::process::exit(0);
    });
}

// ---------------- 托盘 ----------------

fn nid(hwnd: isize, hicon: isize) -> NotifyIconDataW {
    let mut d = NotifyIconDataW {
        cb_size: std::mem::size_of::<NotifyIconDataW>() as u32,
        hwnd,
        uid: 1,
        flags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        callback_message: TRAY_CALLBACK,
        hicon,
        tip: [0; 128],
        state: 0,
        state_mask: 0,
        info: [0; 256],
        timeout_or_version: 0,
        info_title: [0; 64],
        info_flags: 0,
        guid_item: Guid {
            a: 0,
            b: 0,
            c: 0,
            d: [0; 8],
        },
        balloon_icon: 0,
    };
    let tip = wide("RazerControl · 雷蛇驱动");
    d.tip[..tip.len()].copy_from_slice(&tip);
    d
}

unsafe extern "system" fn wnd_proc(hwnd: isize, msg: u32, w: usize, l: isize) -> isize {
    if let Some(&taskbar) = TASKBAR_CREATED.get() {
        if msg == taskbar {
            let d = nid(hwnd, make_icon());
            Shell_NotifyIconW(NIM_ADD, &d);
            return 0;
        }
    }
    if msg == TRAY_CALLBACK {
        let ev = (l as u32) & 0xFFFF;
        if ev == WM_LBUTTONUP || ev == WM_LBUTTONDBLCLK {
            show_main();
        } else if ev == WM_RBUTTONUP {
            unsafe {
                let menu = CreatePopupMenu();
                AppendMenuW(menu, MF_STRING, 1, wide("显示主界面").as_ptr());
                AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
                AppendMenuW(menu, MF_STRING, 2, wide("退出 RazerControl").as_ptr());
                let mut pt = Point { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                SetForegroundWindow(hwnd);
                let cmd = TrackPopupMenu(
                    menu,
                    TPM_RETURNCMD | TPM_RIGHTBUTTON,
                    pt.x,
                    pt.y,
                    0,
                    hwnd,
                    std::ptr::null(),
                );
                PostMessageW(hwnd, 0, 0, 0);
                DestroyMenu(menu);
                match cmd {
                    1 => show_main(),
                    2 => quit_app(),
                    _ => {}
                }
            }
        }
        return 0;
    }
    DefWindowProcW(hwnd, msg, w, l)
}

/// 安装托盘图标（只调用一次）
pub fn install(ctx: egui::Context) {
    let _ = CTX.set(ctx);
    std::thread::spawn(|| unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class_name = wide("RazerControlTrayWnd");
        let wc = WndClassExW {
            cb_size: std::mem::size_of::<WndClassExW>() as u32,
            style: 0,
            wnd_proc: Some(wnd_proc),
            cls_extra: 0,
            wnd_extra: 0,
            instance,
            icon: 0,
            cursor: 0,
            background: 0,
            menu_name: std::ptr::null(),
            class_name: class_name.as_ptr(),
            icon_small: 0,
        };
        RegisterClassExW(&wc);
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide("RazerControlTray").as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            instance,
            std::ptr::null_mut(),
        );
        if hwnd == 0 {
            return;
        }
        let _ = TASKBAR_CREATED.set(RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()));
        let d = nid(hwnd, make_icon());
        Shell_NotifyIconW(NIM_ADD, &d);

        let mut msg: Msg = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Shell_NotifyIconW(NIM_DELETE, &d);
        DestroyWindow(hwnd);
    });
}

/// 托盘气泡提示
pub fn notify(title: &str, text: &str) {
    let title = title.to_string();
    let text = text.to_string();
    std::thread::spawn(move || unsafe {
        let hwnd = FindWindowW(wide("RazerControlTrayWnd").as_ptr(), std::ptr::null());
        if hwnd == 0 {
            return;
        }
        let mut d = nid(hwnd, 0);
        d.flags = NIF_INFO;
        d.info_flags = NIIF_INFO;
        let t = wide(&title);
        let m = wide(&text);
        d.info_title[..t.len()].copy_from_slice(&t);
        d.info[..m.len()].copy_from_slice(&m);
        Shell_NotifyIconW(NIM_MODIFY, &d);
    });
}
