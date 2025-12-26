use libc::{c_char, c_int, c_void};

pub mod ffi {
    use super::*;
    extern "C" {
        pub fn DobbyHook(
            function_address: *mut c_void,
            replace_call: *mut c_void,
            origin_call: *mut *mut c_void,
        ) -> c_int;

        pub fn DobbyUnhook(function_address: *mut c_void) -> c_int;

        pub fn DobbyDestroy(function_address: *mut c_void) -> c_int;

        // Dobby also typically exports these, though we might not use them all
        pub fn DobbyImport(image_name: *const c_char, symbol_name: *const c_char) -> *mut c_void;

        pub fn DobbySymbolResolver(
            image_name: *const c_char,
            symbol_name: *const c_char,
        ) -> *mut c_void;

        #[link_name = "DobbyCodePatch"]
        pub fn CodePatch(address: *mut c_void, buffer: *mut u8, size: u32) -> c_int;
    }

    pub const MemoryOperationError_kMemoryOperationSuccess: c_int = 0;
    pub const MemoryOperationError_kMemoryOperationError: c_int = 1;
    pub const MemoryOperationError_kNotEnough: c_int = 2;
    pub const MemoryOperationError_kNotSupportAllocateExecutableMemory: c_int = 3;
    pub const MemoryOperationError_kNone: c_int = 4;
}
