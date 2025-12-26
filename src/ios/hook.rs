use crate::core::Hachimi;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

type DyldFunc = extern "C" fn(header: *const c_void, vm_addr_slide: isize);

extern "C" {
    fn _dyld_register_func_for_add_image(func: DyldFunc);
}

// Wrapper for dladdr which is not in rust libc for iOS sometimes or needs explicit decl
extern "C" {
    fn dladdr(addr: *const c_void, info: *mut Dl_info) -> c_int;
}

#[repr(C)]
struct Dl_info {
    dli_fname: *const c_char,
    dli_fbase: *mut c_void,
    dli_sname: *const c_char,
    dli_saddr: *mut c_void,
}

extern "C" fn image_callback(header: *const c_void, _slide: isize) {
    let mut info = Dl_info {
        dli_fname: std::ptr::null(),
        dli_fbase: std::ptr::null_mut(),
        dli_sname: std::ptr::null(),
        dli_saddr: std::ptr::null_mut(),
    };

    unsafe {
        if dladdr(header, &mut info) != 0 && !info.dli_fname.is_null() {
            let name_c = CStr::from_ptr(info.dli_fname);
            let name = name_c.to_string_lossy();

            // Log every image for debug
            // log::debug!("Image loaded: {}", name);

            if crate::hachimi_impl::is_il2cpp_lib(&name) {
                log::info!("Found Il2Cpp via dyld: {}", name);

                // Show debug popup as requested
                let msg = format!("Found Il2Cpp:\n{}", name);
                super::show_alert("Hachimi Debug", &msg);

                // Initialize Hachimi core with this lib
                // We pass the header as handle. Note: on iOS symbols_impl might re-dlopen by name.
                // We must use dlopen to get a valid handle for dlsym, using the header pointer directly won't work.
                let handle = libc::dlopen(info.dli_fname, libc::RTLD_LAZY | libc::RTLD_NOLOAD);
                if !handle.is_null() {
                    Hachimi::instance().on_dlopen(&name, handle as usize);

                    // Now that UnityFramework is loaded and handle is set, we can initialize hooks
                    crate::il2cpp::hook::init();
                } else {
                    log::error!("Failed to dlopen detected il2cpp lib: {}", name);
                }
            }
        }
    }
}

pub fn init() {
    unsafe {
        log::info!("Registering dyld image callback...");
        _dyld_register_func_for_add_image(image_callback);
    }
}
