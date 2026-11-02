//! Win32 HID 传输层：设备枚举 / 打开 / Feature Report 收发。
//!
//! 关键发现（逆向自 RzDev_*.sys + 实测）：
//! * 雷蛇设备的控制接口为 HID usage page 0x0001 / usage 0x0002，Feature Report 长度 91 字节。
//! * 该接口被 Synapse 的内核驱动独占 **读** 权限，但仅请求 `GENERIC_WRITE` 可以成功打开，
//!   且 `HidD_GetFeature` 在写句柄上依然可用 —— 这是我们无需卸载 Synapse 就能通信的原因。

use std::ffi::c_void;
use std::ptr;

pub const RAZER_VID: u16 = 0x1532;

pub const GENERIC_WRITE: u32 = 0x4000_0000;
pub const FILE_SHARE_READ_WRITE: u32 = 0x0000_0003;
pub const OPEN_EXISTING: u32 = 3;
pub const INVALID_HANDLE: isize = -1;

const DIGCF_PRESENT: u32 = 0x02;
const DIGCF_DEVICEINTERFACE: u32 = 0x10;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[repr(C)]
struct SpDeviceInterfaceData {
    cb_size: u32,
    interface_class_guid: Guid,
    flags: u32,
    reserved: usize,
}

#[repr(C)]
struct HiddAttributes {
    size: u32,
    vendor_id: u16,
    product_id: u16,
    version_number: u16,
}

#[repr(C)]
struct HidpCaps {
    usage: u16,
    usage_page: u16,
    input_report_byte_length: u16,
    output_report_byte_length: u16,
    feature_report_byte_length: u16,
    reserved: [u16; 17],
    number_link_collection_nodes: u16,
    number_input_button_caps: u16,
    number_input_value_caps: u16,
    number_input_data_indices: u16,
    number_output_button_caps: u16,
    number_output_value_caps: u16,
    number_output_data_indices: u16,
    number_feature_button_caps: u16,
    number_feature_value_caps: u16,
    number_feature_data_indices: u16,
}

impl Default for HidpCaps {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[link(name = "hid")]
extern "system" {
    fn HidD_GetHidGuid(guid: *mut Guid);
    fn HidD_GetAttributes(device: isize, attributes: *mut HiddAttributes) -> i32;
    fn HidD_GetPreparsedData(device: isize, preparsed: *mut *mut c_void) -> i32;
    fn HidD_FreePreparsedData(preparsed: *mut c_void) -> i32;
    fn HidP_GetCaps(preparsed: *mut c_void, caps: *mut HidpCaps) -> i32;
    fn HidD_SetFeature(device: isize, buffer: *mut u8, len: u32) -> i32;
    fn HidD_GetFeature(device: isize, buffer: *mut u8, len: u32) -> i32;
    fn HidD_GetProductString(device: isize, buffer: *mut u16, len: u32) -> i32;
    fn HidD_GetManufacturerString(device: isize, buffer: *mut u16, len: u32) -> i32;
    fn HidD_GetSerialNumberString(device: isize, buffer: *mut u16, len: u32) -> i32;
}

#[link(name = "setupapi")]
extern "system" {
    fn SetupDiGetClassDevsW(
        class_guid: *const Guid,
        enumerator: *const u16,
        parent: isize,
        flags: u32,
    ) -> isize;
    fn SetupDiEnumDeviceInterfaces(
        dev_info: isize,
        dev_info_data: *const c_void,
        interface_class_guid: *const Guid,
        index: u32,
        interface_data: *mut SpDeviceInterfaceData,
    ) -> i32;
    fn SetupDiGetDeviceInterfaceDetailW(
        dev_info: isize,
        interface_data: *const SpDeviceInterfaceData,
        detail: *mut c_void,
        detail_size: u32,
        required_size: *mut u32,
        device_info_data: *mut c_void,
    ) -> i32;
    fn SetupDiDestroyDeviceInfoList(dev_info: isize) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateFileW(
        path: *const u16,
        desired_access: u32,
        share_mode: u32,
        security: *const c_void,
        disposition: u32,
        flags: u32,
        template: isize,
    ) -> isize;
    fn CloseHandle(handle: isize) -> i32;
}

/// 一个雷蛇 HID 接口的描述。
#[derive(Clone, Debug)]
pub struct HidInfo {
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub version: u16,
    pub product: String,
    pub manufacturer: String,
    pub usage_page: u16,
    pub usage: u16,
    pub feature_len: u16,
    pub input_len: u16,
}

impl HidInfo {
    pub fn is_control_interface(&self) -> bool {
        self.vendor_id == RAZER_VID && self.feature_len == 91 && self.usage_page == 0x0001
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn read_wide(ptr: *const u16, max: usize) -> String {
    let mut len = 0usize;
    unsafe {
        while len < max && *ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
    }
}

fn valid_handle(h: isize) -> bool {
    h != INVALID_HANDLE && h != 0
}

/// 枚举系统中所有 HID 接口，返回雷蛇设备相关的条目。
pub fn enumerate_razer() -> Vec<HidInfo> {
    unsafe {
        let mut guid: Guid = std::mem::zeroed();
        HidD_GetHidGuid(&mut guid);

        let dev_info =
            SetupDiGetClassDevsW(&guid, ptr::null(), 0, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE);
        if dev_info == INVALID_HANDLE || dev_info == 0 {
            return Vec::new();
        }

        let mut out = Vec::new();
        let mut index = 0u32;
        loop {
            let mut did = SpDeviceInterfaceData {
                cb_size: std::mem::size_of::<SpDeviceInterfaceData>() as u32,
                interface_class_guid: std::mem::zeroed(),
                flags: 0,
                reserved: 0,
            };
            if SetupDiEnumDeviceInterfaces(dev_info, ptr::null(), &guid, index, &mut did) == 0 {
                break;
            }
            index += 1;

            let mut required: u32 = 0;
            SetupDiGetDeviceInterfaceDetailW(
                dev_info,
                &did,
                ptr::null_mut(),
                0,
                &mut required,
                ptr::null_mut(),
            );
            if required == 0 {
                continue;
            }
            let mut buf = vec![0u8; required as usize];
            // cbSize 字段 = sizeof(SP_DEVICE_INTERFACE_DETAIL_DATA_W) = 8 (x64)
            ptr::write_unaligned(buf.as_mut_ptr() as *mut u32, 8u32);
            if SetupDiGetDeviceInterfaceDetailW(
                dev_info,
                &did,
                buf.as_mut_ptr() as *mut c_void,
                required,
                ptr::null_mut(),
                ptr::null_mut(),
            ) == 0
            {
                continue;
            }
            let path = read_wide(buf.as_ptr().add(4) as *const u16, (required as usize).saturating_sub(4));
            if path.is_empty() {
                continue;
            }
            if let Some(info) = probe(&path) {
                if info.vendor_id == RAZER_VID {
                    out.push(info);
                }
            }
        }
        SetupDiDestroyDeviceInfoList(dev_info);
        out
    }
}

/// 打开设备（只读访问会失败，必须用 GENERIC_WRITE）并读取描述信息。
fn probe(path: &str) -> Option<HidInfo> {
    unsafe {
        let handle = open_raw(path, 0)?;
        let mut attrs = HiddAttributes {
            size: std::mem::size_of::<HiddAttributes>() as u32,
            vendor_id: 0,
            product_id: 0,
            version_number: 0,
        };
        HidD_GetAttributes(handle, &mut attrs);

        let mut caps = HidpCaps::default();
        let mut prep: *mut c_void = ptr::null_mut();
        if HidD_GetPreparsedData(handle, &mut prep) != 0 {
            HidP_GetCaps(prep, &mut caps);
            HidD_FreePreparsedData(prep);
        }

        let mut buf = [0u16; 256];
        let mut product = String::new();
        if HidD_GetProductString(handle, buf.as_mut_ptr(), 512) != 0 {
            product = read_wide(buf.as_ptr(), 255);
        }
        let mut manufacturer = String::new();
        if HidD_GetManufacturerString(handle, buf.as_mut_ptr(), 512) != 0 {
            manufacturer = read_wide(buf.as_ptr(), 255);
        }
        let mut serial = String::new();
        if HidD_GetSerialNumberString(handle, buf.as_mut_ptr(), 512) != 0 {
            serial = read_wide(buf.as_ptr(), 255);
        }
        let _ = serial;

        CloseHandle(handle);

        Some(HidInfo {
            path: path.to_string(),
            vendor_id: attrs.vendor_id,
            product_id: attrs.product_id,
            version: attrs.version_number,
            product,
            manufacturer,
            usage_page: caps.usage_page,
            usage: caps.usage,
            feature_len: caps.feature_report_byte_length,
            input_len: caps.input_report_byte_length,
        })
    }
}

unsafe fn open_raw(path: &str, access: u32) -> Option<isize> {
    let w = wide(path);
    let handle = CreateFileW(
        w.as_ptr(),
        access,
        FILE_SHARE_READ_WRITE,
        ptr::null(),
        OPEN_EXISTING,
        0,
        0,
    );
    if valid_handle(handle) {
        Some(handle)
    } else {
        None
    }
}

/// 已打开的 HID 控制通道。
pub struct HidDevice {
    handle: isize,
    pub path: String,
}

unsafe impl Send for HidDevice {}

impl HidDevice {
    /// 以写权限打开控制接口（读权限被厂商驱动独占）。
    pub fn open(path: &str) -> Option<Self> {
        unsafe {
            let handle = open_raw(path, GENERIC_WRITE)?;
            Some(HidDevice {
                handle,
                path: path.to_string(),
            })
        }
    }

    pub fn set_feature(&self, buf: &[u8]) -> bool {
        unsafe { HidD_SetFeature(self.handle, buf.as_ptr() as *mut u8, buf.len() as u32) != 0 }
    }

    pub fn get_feature(&self, buf: &mut [u8]) -> bool {
        unsafe { HidD_GetFeature(self.handle, buf.as_mut_ptr(), buf.len() as u32) != 0 }
    }
}

impl Drop for HidDevice {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}
