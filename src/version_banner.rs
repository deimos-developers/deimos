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

use crate::VERSION_UTF16;
use crate::syringe_api::Registers;

/// Address of the instruction inside `GScreenClass::DrawText` that Phobos
/// patches. The hook site is shared with Phobos; `syringe-rs` chains multiple
/// hook procs at the same address.
const HOOK_ADDR: u32 = 0x004F_4583;
const HOOK_SIZE: u32 = 6;

/// `Drawing::GetTextDimensions` (`__fastcall`), 0x4A59E0 in YRpp.
const GET_TEXT_DIMENSIONS: u32 = 0x004A_59E0;
/// `Fancy_Text_Print_Wide` (`__cdecl`), 0x4A60E0 in YRpp.
const FANCY_TEXT_PRINT_WIDE: u32 = 0x004A_60E0;
/// `DSurface::Composite` (`DSurface*` global), 0x88731C in YRpp.
const COMPOSITE: u32 = 0x0088_731C;

/// YRpp `COLOR_BLACK`.
const COLOR_BLACK: u32 = 0x0000;
/// YRpp `COLOR_RED` (RGB 5-6-5).
const COLOR_RED: u32 = 0xF800;
/// YRpp `TextPrintType::NoShadow`.
const TEXT_NO_SHADOW: i32 = 0x10;
/// Gap between the banner and the right screen edge, as in Phobos.
const MARGIN_X: i32 = 10;
/// Padding between the text and the filled background rectangle.
const TEXT_PADDING: i32 = 5;
/// Top offset when Phobos is not loaded.
const DEFAULT_TEXT_Y: i32 = 5;
/// Top offset when Phobos is loaded; keeps Deimos below Phobos' warning.
const PHOBOS_TEXT_Y: i32 = 50;

#[repr(C)]
#[derive(Clone, Copy)]
struct Point2D {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RectangleStruct {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

type GetTextDimensionsFn = unsafe extern "fastcall" fn(
    out: *mut RectangleStruct,
    text: *const u16,
    location: Point2D,
    flags: u32,
    margin_x: i32,
    margin_y: i32,
) -> *mut RectangleStruct;

type FancyTextPrintWideFn = unsafe extern "C" fn(
    ret_buffer: *mut Point2D,
    text: *const u16,
    surface: *mut core::ffi::c_void,
    bounds: *const RectangleStruct,
    location: *const Point2D,
    fore_color: u32,
    back_color: u32,
    flags: i32,
) -> *mut Point2D;

/// `Surface::GetWidth` / `Surface::GetHeight` (MSVC vtable slots 31 and 32;
/// the vtable starts with one destructor slot).
type GetWidthFn = unsafe extern "thiscall" fn(surface: *mut core::ffi::c_void) -> i32;
type GetHeightFn = unsafe extern "thiscall" fn(surface: *mut core::ffi::c_void) -> i32;
/// `Surface::FillRect` (MSVC vtable slot 5). The return value is unused.
type FillRectFn = unsafe extern "thiscall" fn(
    surface: *mut core::ffi::c_void,
    rect: *const RectangleStruct,
    color: u32,
);

#[inline]
fn get_text_dimensions(
    out: *mut RectangleStruct,
    text: *const u16,
    location: Point2D,
    flags: u32,
    margin_x: i32,
    margin_y: i32,
) -> *mut RectangleStruct {
    // SAFETY: the address is the documented engine entry point and the
    // signature matches YRpp's `Drawing::GetTextDimensions`.
    let function: GetTextDimensionsFn =
        unsafe { core::mem::transmute(GET_TEXT_DIMENSIONS as usize) };
    unsafe { function(out, text, location, flags, margin_x, margin_y) }
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn fancy_text_print_wide(
    ret_buffer: *mut Point2D,
    text: *const u16,
    surface: *mut core::ffi::c_void,
    bounds: *const RectangleStruct,
    location: *const Point2D,
    fore_color: u32,
    back_color: u32,
    flags: i32,
) -> *mut Point2D {
    // SAFETY: the address is the documented engine entry point and the
    // signature matches YRpp's `Fancy_Text_Print_Wide` (minus the varargs,
    // which are unused here).
    let function: FancyTextPrintWideFn =
        unsafe { core::mem::transmute(FANCY_TEXT_PRINT_WIDE as usize) };
    unsafe {
        function(
            ret_buffer, text, surface, bounds, location, fore_color, back_color, flags,
        )
    }
}

#[inline]
unsafe fn composite_surface() -> *mut core::ffi::c_void {
    // SAFETY: 0x88731C is the engine's `DSurface::Composite` pointer.
    unsafe { *(COMPOSITE as *const *mut core::ffi::c_void) }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleA(module_name: *const u8) -> *mut core::ffi::c_void;
}

/// Whether the game has loaded Phobos' overlay DLL.
fn phobos_loaded() -> bool {
    // SAFETY: the name is a valid NUL-terminated string; the call has no
    // other preconditions.
    unsafe { !GetModuleHandleA(c"Phobos.dll".as_ptr().cast()).is_null() }
}

#[inline]
unsafe fn fill_rect(surface: *mut core::ffi::c_void, rect: *const RectangleStruct, color: u32) {
    // SAFETY: `surface` is the engine's composite DSurface and `rect` is a
    // valid rectangle. FillRect is vtable slot 5 in YRpp's Surface.h.
    unsafe {
        let vtable = *(surface as *const *const usize);
        let fill: FillRectFn = core::mem::transmute(*vtable.add(5));
        fill(surface, rect, color);
    }
}

#[inline]
unsafe fn surface_dimensions(surface: *mut core::ffi::c_void) -> (i32, i32) {
    // SAFETY: `surface` is the engine's composite `DSurface`; its vtable
    // starts with one MSVC destructor slot, making GetWidth/GetHeight slots
    // 31/32 as declared in YRpp's Surface.h.
    unsafe {
        let vtable = *(surface as *const *const usize);
        let get_width: GetWidthFn = core::mem::transmute(*vtable.add(31));
        let get_height: GetHeightFn = core::mem::transmute(*vtable.add(32));
        (get_width(surface), get_height(surface))
    }
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
        let (width, height) = surface_dimensions(surface);
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
        let background = RectangleStruct {
            x: width - wanted.width - MARGIN_X,
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
        // The engine takes the first argument as a `const Point2D&` but uses
        // it as an output buffer, so it must stay on the writable stack (a
        // temporary would be placed in read-only data).
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

crate::declhook!(HOOK_ADDR, Deimos_GScreenClass_DrawText, HOOK_SIZE);

// Also declare the executable for loaders that do not run a handshake.
crate::declhost!(gamemd, 0);
