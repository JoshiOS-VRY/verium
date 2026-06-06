//! FFI to portable scrypt² (`verium-pool/packages/hashing/src/scrypt2.c`).

use std::ptr;

/// Scratchpad size for N=2^20 scrypt (matches `SCRYPT_SCRATCHPAD_SIZE` in scrypt2.c).
#[allow(dead_code)]
pub const SCRATCHPAD_SIZE: usize = 134_218_239;

extern "C" {
    fn scrypt2_buffer_alloc() -> *mut u8;
    fn scrypt2_hash(input: *const std::ffi::c_void, output: *mut u8, scratchpad: *mut u8);
}

/// Per-thread scratch buffer (~128 MiB). Drop frees via `libc::free` equivalent.
pub struct Scratchpad {
    ptr: *mut u8,
}

impl Scratchpad {
    pub fn try_new() -> Option<Self> {
        let ptr = unsafe { scrypt2_buffer_alloc() };
        if ptr.is_null() {
            return None;
        }
        Some(Self { ptr })
    }

    pub fn new() -> Self {
        Self::try_new().expect("scrypt2 scratchpad allocation failed (out of memory)")
    }

    pub fn hash(&self, header: &[u8; 80]) -> [u8; 32] {
        let mut out = [0u8; 32];
        unsafe {
            scrypt2_hash(
                header.as_ptr() as *const std::ffi::c_void,
                out.as_mut_ptr(),
                self.ptr,
            );
        }
        out
    }
}

impl Drop for Scratchpad {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                libc::free(self.ptr as *mut libc::c_void);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

unsafe impl Send for Scratchpad {}
