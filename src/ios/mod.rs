use std::os::raw::{c_char, c_void};

// use crate::hachimi_impl as core_hachimi_impl;

pub mod game_impl;
pub mod gui_impl;
pub mod hachimi_impl;
pub mod hook;
pub mod input;
pub mod interceptor_impl;
pub mod log_impl;
pub mod renderer;
pub mod symbols_impl;
pub mod utils;

use objc::runtime::{Class, Object, Sel};
#[allow(unused_imports)]
use objc::{class, Message};
use objc::{msg_send, sel, sel_impl};

#[cfg(target_os = "ios")]
#[link(name = "c++")]
extern "C" {}

#[ctor::ctor]
fn init() {
    // iOS dylib entry point
    if crate::core::Hachimi::init() {
        log::info!("Hachimi initialized on iOS");

        // hook::init(); // Moved to image_callback to ensure UnityFramework is loaded
        // gui_impl::init(); // DISABLED per user request to debug crash

        // Show alert and unlock FPS on main thread
        std::thread::spawn(|| {
            // Give the app some time to initialize UI and Unity
            std::thread::sleep(std::time::Duration::from_secs(7));

            unsafe {
                unlock_fps_on_main_thread();

                let pkg = game_impl::get_package_name();
                let region = game_impl::get_region(&pkg);
                let data_dir = game_impl::get_data_dir(&pkg);

                let msg = format!(
                    "Injection Successful!\nFPS Unlocked: 240\nPkg: {}\nRegion: {:?}\nData: {:?}",
                    pkg, region, data_dir
                );
                let _ = show_alert("Hachimi Edge", &msg);
            }
        });
    }
}

unsafe fn unlock_fps_on_main_thread() {
    let superclass = Class::get("NSObject").unwrap();
    if Class::get("HachimiFpsHelper").is_none() {
        let mut decl = objc::declare::ClassDecl::new("HachimiFpsHelper", superclass).unwrap();
        decl.add_method(
            sel!(unlockFps),
            unlock_fps_impl as extern "C" fn(&Object, Sel),
        );
        decl.register();
    }

    let cls = Class::get("HachimiFpsHelper").unwrap();
    let helper: *mut Object = msg_send![cls, new];
    let _: () = msg_send![helper, performSelectorOnMainThread:sel!(unlockFps) withObject:std::ptr::null_mut::<Object>() waitUntilDone:false];
}

extern "C" fn unlock_fps_impl(_this: &Object, _cmd: Sel) {
    unsafe {
        // Auto-Unlock FPS to 240
        let func_addr = crate::il2cpp::api::il2cpp_resolve_icall(
            c"UnityEngine.Application::set_targetFrameRate(System.Int32)".as_ptr(),
        );
        if func_addr != 0 {
            let func: extern "C" fn(i32) = std::mem::transmute(func_addr);
            func(240);
            crate::core::Hachimi::instance()
                .target_fps
                .store(240, std::sync::atomic::Ordering::Relaxed);
            log::info!("Auto-set FPS to 240 on Main Thread (ObjC)");
        } else {
            log::error!("Failed to resolve set_targetFrameRate");
        }
    }
}

pub(crate) unsafe fn show_alert(title: &str, message: &str) {
    // Create a class helper to handle the alert on main thread
    let superclass = Class::get("NSObject").unwrap();
    if Class::get("HachimiAlertHelper").is_none() {
        let mut decl = objc::declare::ClassDecl::new("HachimiAlertHelper", superclass).unwrap();
        // Use *mut Object for arguments to avoid lifetime issues in matching Fn signatures
        decl.add_method(
            sel!(showAlert:),
            show_alert_impl as extern "C" fn(&Object, Sel, *mut Object),
        );
        decl.register();
    }

    let cls = Class::get("HachimiAlertHelper").unwrap();
    let helper: *mut Object = msg_send![cls, new];

    // Use a simple delimiter
    let code = format!("{}|||{}", title, message);
    let str_cls = Class::get("NSString").unwrap();
    let arg: *mut Object =
        msg_send![str_cls, stringWithUTF8String: std::ffi::CString::new(code).unwrap().as_ptr()];

    let _: () = msg_send![helper, performSelectorOnMainThread:sel!(showAlert:) withObject:arg waitUntilDone:false];
}

extern "C" fn show_alert_impl(_this: &Object, _cmd: Sel, arg: *mut Object) {
    unsafe {
        let arg_str: *const std::os::raw::c_char = msg_send![arg, UTF8String];
        let rust_str = std::ffi::CStr::from_ptr(arg_str).to_string_lossy();

        // Simple delimiter parsing
        let parts: Vec<&str> = rust_str.split("|||").collect();
        let (t, m) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("Hachimi", rust_str.as_ref())
        };

        let title_cls = Class::get("NSString").unwrap();
        let msg_cls = Class::get("NSString").unwrap();
        let t_obj: *mut Object =
            msg_send![title_cls, stringWithUTF8String: std::ffi::CString::new(t).unwrap().as_ptr()];
        let m_obj: *mut Object =
            msg_send![msg_cls, stringWithUTF8String: std::ffi::CString::new(m).unwrap().as_ptr()];

        let alert_cls = Class::get("UIAlertController").unwrap();
        let alert: *mut Object =
            msg_send![alert_cls, alertControllerWithTitle:t_obj message:m_obj preferredStyle:1]; // 1 = UIAlertControllerStyleAlert

        let action_cls = Class::get("UIAlertAction").unwrap();
        let style = 0; // UIAlertActionStyleDefault
        let handler = std::ptr::null_mut::<c_void>(); // No handler
        let ok_str: *mut Object = msg_send![title_cls, stringWithUTF8String: std::ffi::CString::new("OK").unwrap().as_ptr()];
        let action: *mut Object =
            msg_send![action_cls, actionWithTitle:ok_str style:style handler:handler];

        let _: () = msg_send![alert, addAction:action];

        let app_cls = Class::get("UIApplication").unwrap();
        let shared_app: *mut Object = msg_send![app_cls, sharedApplication];
        let key_window: *mut Object = msg_send![shared_app, keyWindow];

        if !key_window.is_null() {
            let root_vc: *mut Object = msg_send![key_window, rootViewController];
            if !root_vc.is_null() {
                let _: () = msg_send![root_vc, presentViewController:alert animated:true completion:std::ptr::null_mut::<c_void>()];
            }
        }
    }
}
