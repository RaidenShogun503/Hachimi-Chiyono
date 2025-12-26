use crate::core::game::Region;
use objc::runtime::{Class, Object};
use objc::{class, msg_send, sel, sel_impl};
use std::path::PathBuf;

pub fn get_package_name() -> String {
    unsafe {
        let cls = Class::get("NSBundle").unwrap();
        let bundle: *mut Object = msg_send![cls, mainBundle];
        let bundle_id: *mut Object = msg_send![bundle, bundleIdentifier];

        if bundle_id.is_null() {
            return "unknown".to_string();
        }

        let utf8_str: *const std::os::raw::c_char = msg_send![bundle_id, UTF8String];
        let bytes = std::ffi::CStr::from_ptr(utf8_str).to_bytes();
        String::from_utf8_lossy(bytes).into_owned()
    }
}

pub fn get_region(package_name: &str) -> Region {
    match package_name {
        "jp.co.cygames.umamusume" => Region::Japan,
        "com.komoe.kmumamusumegp" | "com.komoe.umamusumeofficial" => Region::Taiwan,
        "com.kakaogames.umamusume" => Region::Korea,
        "com.bilibili.umamusu" => Region::China,
        "com.cygames.umamusume" => Region::Global,
        _ => Region::Unknown,
    }
}

pub fn get_data_dir(package_name: &str) -> PathBuf {
    // On iOS, we sandbox to Documents directly often.
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join("Documents").join("hachimi");
    }
    PathBuf::from(".").join("hachimi")
}
