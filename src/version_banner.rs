//! Experimental in-game version banner, modeled on Phobos' testing-build
//! overlay (`Phobos/src/Phobos.cpp`, `GScreenClass_DrawText`).
//!
//! The module hooks `GScreenClass::DrawText` (0x4F4583) and draws
//! [`crate::VERSION`] in the top-right corner of the composite surface. It is
//! behind the `version-banner` Cargo feature because the engine addresses
//! come from Phobos/YRpp and must match the exact `gamemd.exe` build. The
//! loader does not verify the executable, so a different build can call into
//! unrelated code.
//!
//! The banner fills a black background like Phobos and draws the version in
//! red on top of it. Its vertical position depends on whether `Phobos.dll` is
//! loaded: Phobos' testing-build warning occupies the top rows, so Deimos
//! moves below it when Phobos is present and stays at the top otherwise.

use core::ffi::c_void;
use std::sync::OnceLock;

use crate::VERSION_UTF16;
use crate::engine::drawing::{
    COLOR_BLACK, COLOR_RED, Point2D, RectangleStruct, TEXT_NO_SHADOW, composite_surface,
    fancy_text_print_wide, fill_rect, get_text_dimensions, surface_size,
};
use crate::engine::hooks::{DRAW_TEXT, DRAW_TEXT_SIZE};
use crate::syringe_api::Registers;

/// Gap between the banner and the right screen edge, as in Phobos.
const MARGIN_X: i32 = 10;
/// Padding between the text and the filled background rectangle.
const TEXT_PADDING: i32 = 5;
/// Top offset when Phobos is not loaded.
const DEFAULT_TEXT_Y: i32 = 5;
/// Top offset when Phobos is loaded; keeps Deimos below Phobos' warning.
const PHOBOS_TEXT_Y: i32 = 50;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleA(module_name: *const u8) -> *mut c_void;
}

/// Cached result of the Phobos DLL probe. Phobos is loaded by the loader
/// before the game starts, so the first draw is authoritative; caching avoids
/// a `GetModuleHandleA` call every frame.
static PHOBOS_LOADED: OnceLock<bool> = OnceLock::new();

/// Whether the game has loaded Phobos' overlay DLL.
fn phobos_loaded() -> bool {
    *PHOBOS_LOADED.get_or_init(|| {
        // SAFETY: the name is a valid NUL-terminated string; the call has no
        // other preconditions.
        unsafe { !GetModuleHandleA(c"Phobos.dll".as_ptr().cast()).is_null() }
    })
}

/// Draws [`crate::VERSION`] in the top-right corner of the game screen.
fn draw_version() {
    // SAFETY: all pointers are read from the engine's own globals and passed
    // straight back to engine functions; nothing is dereferenced here.
    unsafe {
        let surface = composite_surface();
        if surface.is_null() {
            return;
        }
        let (width, height) = surface_size(surface);
        if width <= 0 || height <= 0 {
            return;
        }

        let mut wanted = RectangleStruct {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        get_text_dimensions(
            &mut wanted,
            VERSION_UTF16.as_ptr(),
            Point2D { x: 0, y: 0 },
            0,
            0,
            0,
        );

        let text_y = if phobos_loaded() {
            PHOBOS_TEXT_Y
        } else {
            DEFAULT_TEXT_Y
        };
        let background_x = (width - wanted.width - MARGIN_X).max(MARGIN_X);
        let background = RectangleStruct {
            x: background_x,
            y: text_y - TEXT_PADDING,
            width: wanted.width + TEXT_PADDING * 2,
            height: wanted.height + TEXT_PADDING * 2,
        };
        fill_rect(surface, &background, COLOR_BLACK);

        let location = Point2D {
            x: background.x + TEXT_PADDING,
            y: text_y,
        };
        let bounds = RectangleStruct {
            x: 0,
            y: 0,
            width,
            height,
        };
        // The engine uses the first argument as an output buffer, so keep it
        // on the writable stack.
        let mut ret_buffer = Point2D { x: 0, y: 0 };
        fancy_text_print_wide(
            &mut ret_buffer,
            VERSION_UTF16.as_ptr(),
            surface,
            &bounds,
            &location,
            COLOR_RED,
            0,
            TEXT_NO_SHADOW,
        );
    }
}

/// Hook proc for `GScreenClass::DrawText`; returning `0` lets the engine
/// continue with the original text drawing.
#[unsafe(no_mangle)]
pub extern "C" fn Deimos_GScreenClass_DrawText(_registers: *mut Registers) -> u32 {
    draw_version();
    0
}

crate::declhook!(DRAW_TEXT, Deimos_GScreenClass_DrawText, DRAW_TEXT_SIZE);
