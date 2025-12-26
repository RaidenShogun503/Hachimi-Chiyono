use std::ffi::c_void;
use std::ffi::CString;

pub unsafe fn dlsym(handle: *mut c_void, name: &str) -> usize {
    let cname = CString::new(name).unwrap();

    // Try provided handle first if it's not null
    if !handle.is_null() {
        let sym = libc::dlsym(handle, cname.as_ptr());
        if !sym.is_null() {
            return sym as usize;
        }
    }

    // Fallback to RTLD_DEFAULT (global search)
    let sym = libc::dlsym(libc::RTLD_DEFAULT, cname.as_ptr());
    if !sym.is_null() {
        return sym as usize;
    }

    0
}

pub fn get_symbol(module: &str, symbol: &str) -> Option<*mut c_void> {
    unsafe {
        // RTLD_DEFAULT is -2 on macOS/iOS usually, or just use dlopen(NULL)
        // using libc::RTLD_DEFAULT might require enabling non-standard features or casting.
        // Let's try opening the specific module if provided, or main process.

        let handle = if module.is_empty() {
            libc::dlopen(std::ptr::null(), libc::RTLD_LAZY)
        } else {
            let m = CString::new(module).ok()?;
            libc::dlopen(m.as_ptr(), libc::RTLD_LAZY)
        };

        if handle.is_null() {
            return None;
        }

        let s = CString::new(symbol).ok()?;
        let sym = libc::dlsym(handle, s.as_ptr());

        // We shouldn't close the handle if it's GLOBAL/main, but dlopen returns a new ref count.
        // For simplicity in this stubs:
        // libc::dlclose(handle);

        if sym.is_null() {
            None
        } else {
            Some(sym)
        }
    }
}
