//! Deimos, a Rust hook host for Command & Conquer: Yuri's Revenge.
//!
//! The crate is a `cdylib` DLL that the Syringe loader can inject. It exposes
//! the Syringe ABI from [`syringe_api`] plus the optional entry points the
//! loader looks for (`SyringeHandshake`, the feature-flag exports and
//! `DllMain`).
//!
//! The version banner that Phobos draws in the top-right corner of the game
//! screen is implemented by the `version-banner` feature. It is disabled by
//! default because it patches engine addresses that must be verified against
//! the exact `gamemd.exe` build being used.

pub mod syringe_api;

#[cfg(all(windows, feature = "version-banner"))]
mod version_banner;

pub use syringe_api::{HookDecl, HookProc, HostDecl, Registers, SyringeHandshakeInfo};

/// Human-readable version, shared by the debug log and the in-game banner.
pub const VERSION: &str = concat!("Deimos ", env!("CARGO_PKG_VERSION"));

/// [`VERSION`] followed by the NUL terminator expected by Win32.
pub const VERSION_MESSAGE: &str = concat!("Deimos ", env!("CARGO_PKG_VERSION"), "\0");

const fn ascii_to_utf16_with_nul<const N: usize>(text: &str) -> [u16; N] {
    let bytes = text.as_bytes();
    assert!(
        N == bytes.len() + 1,
        "the UTF-16 buffer must hold the text plus a NUL"
    );
    let mut buffer = [0u16; N];
    let mut index = 0;
    while index < bytes.len() {
        assert!(bytes[index] < 0x80, "the version string must be ASCII");
        buffer[index] = bytes[index] as u16;
        index += 1;
    }
    buffer
}

/// [`VERSION`] as a NUL-terminated UTF-16 string for the engine's text APIs.
pub const VERSION_UTF16: [u16; VERSION.len() + 1] =
    ascii_to_utf16_with_nul::<{ VERSION.len() + 1 }>(VERSION);

/// The empty declaration that marks this DLL as a hook host even before any
/// real hook is added. The loader skips entries without a name.
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".syhks00")]
pub static SYRINGE_HOOK_DECL: HookDecl = HookDecl::new(0, 0, core::ptr::null());

/// `S_OK`.
const S_OK: i32 = 0;

/// Optional handshake export. `syringe-rs` calls it with `--handshakes`;
/// the C++ Syringe loader imports it unconditionally.
///
/// The handshake only reports the Deimos version; it never rejects a host.
/// Whether the executable is accepted is left to the loader.
///
/// # Safety
///
/// `info` must be null or point to a valid `SyringeHandshakeInfo` whose
/// `message` buffer has `cch_message` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SyringeHandshake(info: *mut SyringeHandshakeInfo) -> i32 {
    if info.is_null() {
        return S_OK;
    }
    // SAFETY: the caller guarantees a valid, writable `SyringeHandshakeInfo`.
    let info = unsafe { &mut *info };
    info.cb_size = core::mem::size_of::<SyringeHandshakeInfo>() as i32;
    // SAFETY: `write_message` honours `cch_message` and `message`.
    unsafe { info.write_message(VERSION) };
    S_OK
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OutputDebugStringA(text: *const u8);
}

/// Reports the version through the Syringe log (`OutputDebugStringA`).
#[cfg(windows)]
#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
    _module: *mut core::ffi::c_void,
    reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if reason == DLL_PROCESS_ATTACH {
        // SAFETY: the string is a valid NUL-terminated literal.
        unsafe { OutputDebugStringA(VERSION_MESSAGE.as_ptr()) };
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constants_agree() {
        assert_eq!(
            VERSION_MESSAGE.as_bytes(),
            [VERSION.as_bytes(), b"\0"].concat()
        );
        let utf16: Vec<u16> = VERSION.encode_utf16().collect();
        assert_eq!(&VERSION_UTF16[..utf16.len()], &utf16[..]);
        assert_eq!(VERSION_UTF16[utf16.len()], 0);
    }

    #[test]
    fn handshake_writes_the_version() {
        let mut buffer = [0u8; 32];
        let mut info = SyringeHandshakeInfo {
            cb_size: 0,
            num_hooks: 0,
            checksum: 0,
            exe_filesize: 0,
            exe_timestamp: 0,
            exe_crc: 0,
            cch_message: buffer.len() as i32,
            message: buffer.as_mut_ptr().cast(),
        };
        // SAFETY: `info` and its message buffer are valid for this call.
        let result = unsafe { SyringeHandshake(&mut info) };
        assert_eq!(result, S_OK);
        assert_eq!(
            info.cb_size as usize,
            core::mem::size_of::<SyringeHandshakeInfo>()
        );
        assert_eq!(
            &buffer[..VERSION.len()],
            VERSION.as_bytes(),
            "version should be copied into the handshake buffer"
        );
        assert_eq!(buffer[VERSION.len()], 0);
        assert_eq!(buffer[VERSION.len() + 1], 0);
    }
}
