#![cfg(windows)]
//! Deimos: a minimal Rust DLL for syringe-rs.
//!
//! The `.syhks00` section makes syringe-rs recognize this DLL as a hook host.
//! It contains one empty hook declaration, so there are no patches to apply.
//! `DllMain` emits the DLL version through the Windows debug output stream; the
//! debugger (syringe-rs) forwards it to `syringe-rs.log`.

use core::ffi::{c_char, c_void};

const DLL_PROCESS_ATTACH: u32 = 1;
const S_OK: i32 = 0;

/// Version banner reported through the debugger log.
const VERSION_MESSAGE: &str = concat!("Deimos ", env!("CARGO_PKG_VERSION"), "\0");

/// Matches `hookdecl` in Syringe's public `Syringe.h` (16 bytes, align 16).
#[repr(C, align(16))]
pub struct HookDecl {
    pub hook_addr: u32,
    pub hook_size: u32,
    pub hook_name: *const c_char,
}

// SAFETY: only immutable bytes are shared.
unsafe impl Sync for HookDecl {}

/// A single empty hook declaration keeps the section present in the PE image.
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".syhks00")]
pub static SYRINGE_HOOK_DECL: HookDecl = HookDecl {
    hook_addr: 0,
    hook_size: 0,
    hook_name: core::ptr::null(),
};

/// Matches `SyringeHandshakeInfo` in Syringe's public `Syringe.h`.
#[repr(C)]
pub struct SyringeHandshakeInfo {
    pub cb_size: i32,
    pub num_hooks: i32,
    pub checksum: u32,
    pub exe_filesize: u32,
    pub exe_timestamp: u32,
    pub exe_crc: u32,
    pub cch_message: i32,
    pub message: *mut c_char,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn OutputDebugStringA(lp_output_string: *const u8);
}

/// Optional handshake export. syringe-rs only calls it with `--handshakes`.
///
/// # Safety
///
/// `info` must be null or point to a valid `SyringeHandshakeInfo`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SyringeHandshake(info: *mut SyringeHandshakeInfo) -> i32 {
    if !info.is_null() {
        // Keep the ABI simple: report success and leave the message empty.
        // SAFETY: the caller (Syringe) passes a valid SyringeHandshakeInfo.
        let info = unsafe { &mut *info };
        info.cb_size = core::mem::size_of::<SyringeHandshakeInfo>() as i32;
        if info.cch_message > 0 && !info.message.is_null() {
            unsafe { *info.message = 0 };
        }
    }
    S_OK
}

/// Standard DLL entry point.
#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
    _hinstance: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            OutputDebugStringA(VERSION_MESSAGE.as_ptr());
        }
    }
    1 // TRUE
}
