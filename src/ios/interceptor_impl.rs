use crate::core::{interceptor::HookHandle, Error};
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

pub unsafe fn hook(orig_addr: usize, hook_addr: usize) -> Result<usize, Error> {
    if orig_addr == 0 || hook_addr == 0 {
        return Err(Error::HookingError(
            "Invalid address: orig_addr or hook_addr is null".to_string(),
        ));
    }

    // Dynamic resolution of MSHookFunction to avoid link-time dependency on CydiaSubstrate
    let symbol_name = CString::new("MSHookFunction").unwrap();
    let mshook_ptr = libc::dlsym(libc::RTLD_DEFAULT, symbol_name.as_ptr());

    if mshook_ptr.is_null() {
        return Err(Error::HookingError(
            "MSHookFunction not found. Ensure ElleKit/Substitute is loaded.".to_string(),
        ));
    }

    let mshook_func: extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_void) =
        std::mem::transmute(mshook_ptr);

    let mut trampoline: *mut c_void = std::ptr::null_mut();

    mshook_func(
        orig_addr as *mut c_void,
        hook_addr as *mut c_void,
        &mut trampoline,
    );

    if !trampoline.is_null() {
        Ok(trampoline as usize)
    } else {
        // If trampoline is null, the hook might have failed or the function is too small/complex.
        // Returning 0 would cause EXC_BAD_ACCESS if the caller tries to use it.
        Err(Error::HookingError(format!(
            "MSHookFunction failed to create trampoline for 0x{:x}",
            orig_addr
        )))
    }
}

pub unsafe fn unhook(hook: &HookHandle) -> Result<(), Error> {
    // ElleKit/Substrate doesn't have a standard "unhook" API exposed easily
    // without keeping track of everything manually or using internal APIs.
    // For now, we leave this as a no-op or TODO.
    Ok(())
}

pub unsafe fn get_vtable_from_instance(instance_addr: usize) -> *mut usize {
    std::ptr::null_mut()
}

pub unsafe fn hook_vtable(
    vtable: *mut usize,
    vtable_index: usize,
    hook_addr: usize,
) -> Result<HookHandle, Error> {
    // Return a dummy HookHandle
    Ok(HookHandle {
        orig_addr: 0,
        trampoline_addr: 0,
        hook_type: crate::core::interceptor::HookType::Vtable,
    })
}

pub unsafe fn unhook_vtable(hook: &HookHandle) -> Result<(), Error> {
    Ok(())
}

pub unsafe fn find_symbol_by_name(module: &str, symbol: &str) -> Result<usize, Error> {
    crate::ios::symbols_impl::get_symbol(module, symbol)
        .map(|ptr| ptr as usize)
        .ok_or_else(|| Error::SymbolNotFound(module.to_owned(), symbol.to_owned()))
}
