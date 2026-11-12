//! Drawing primitives exposed by the engine.
//!
//! The calling conventions and vtable slots mirror YRpp's `Surface.h` and the
//! `Drawing` namespace. They are specific to the 32-bit MSVC build of Yuri's
//! Revenge, so every address and slot must be re-checked when targeting a
//! different executable.

use core::ffi::c_void;

/// YRpp `Point2D`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Point2D {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

/// YRpp `RectangleStruct`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RectangleStruct {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
}

/// YRpp `COLOR_BLACK` (RGB 5-6-5).
pub(crate) const COLOR_BLACK: u32 = 0x0000;
/// YRpp `COLOR_WHITE` (RGB 5-6-5).
#[cfg(feature = "dev-console")]
pub(crate) const COLOR_WHITE: u32 = 0xFFFF;
/// YRpp `COLOR_RED` (RGB 5-6-5).
#[cfg(feature = "version-banner")]
pub(crate) const COLOR_RED: u32 = 0xF800;
/// YRpp `TextPrintType::NoShadow`.
pub(crate) const TEXT_NO_SHADOW: i32 = 0x10;

/// `DSurface::Composite` (`DSurface*` global), 0x88731C in YRpp.
const COMPOSITE: u32 = 0x0088_731C;
/// `Drawing::GetTextDimensions` (`__fastcall`), 0x4A59E0 in YRpp.
#[cfg(feature = "version-banner")]
const GET_TEXT_DIMENSIONS: u32 = 0x004A_59E0;
/// `Fancy_Text_Print_Wide` (`__cdecl`), 0x4A60E0 in YRpp.
const FANCY_TEXT_PRINT_WIDE: u32 = 0x004A_60E0;

#[cfg(feature = "version-banner")]
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
    surface: *mut c_void,
    bounds: *const RectangleStruct,
    location: *const Point2D,
    fore_color: u32,
    back_color: u32,
    flags: i32,
) -> *mut Point2D;

/// `Surface::GetWidth` / `Surface::GetHeight` (MSVC vtable slots 31 and 32;
/// the vtable starts with one destructor slot).
type GetWidthFn = unsafe extern "thiscall" fn(surface: *mut c_void) -> i32;
type GetHeightFn = unsafe extern "thiscall" fn(surface: *mut c_void) -> i32;
/// `Surface::FillRect` (MSVC vtable slot 5). The return value is unused.
type FillRectFn =
    unsafe extern "thiscall" fn(surface: *mut c_void, rect: *const RectangleStruct, color: u32);

/// Reads the engine's composite surface pointer.
///
/// # Safety
///
/// The engine must have initialized `DSurface::Composite`.
#[inline]
pub(crate) unsafe fn composite_surface() -> *mut c_void {
    // SAFETY: 0x88731C is the engine's `DSurface::Composite` pointer.
    unsafe { *(COMPOSITE as *const *mut c_void) }
}

/// Measures `text` with `Drawing::GetTextDimensions`.
///
/// # Safety
///
/// `out` must point to a writable `RectangleStruct` and `text` must be a
/// NUL-terminated UTF-16 string.
#[cfg(feature = "version-banner")]
#[inline]
pub(crate) unsafe fn get_text_dimensions(
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

/// Draws `text` with `Fancy_Text_Print_Wide`.
///
/// The first argument is declared as `const Point2D&` in YRpp, but the engine
/// uses it as an output buffer. Callers must pass a writable stack location,
/// not a temporary that the compiler may place in read-only data.
///
/// # Safety
///
/// All pointers must be valid for the duration of the call.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn fancy_text_print_wide(
    ret_buffer: *mut Point2D,
    text: *const u16,
    surface: *mut c_void,
    bounds: &RectangleStruct,
    location: &Point2D,
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

/// Fills `rect` on `surface` with `color`.
///
/// # Safety
///
/// `surface` must be a valid engine surface and `rect` a valid rectangle.
#[inline]
pub(crate) unsafe fn fill_rect(surface: *mut c_void, rect: &RectangleStruct, color: u32) {
    // SAFETY: `surface` is an engine surface and FillRect is vtable slot 5 in
    // YRpp's Surface.h.
    unsafe {
        let vtable = *(surface as *const *const usize);
        let fill: FillRectFn = core::mem::transmute(*vtable.add(5));
        fill(surface, rect, color);
    }
}

/// Returns the surface's width and height.
///
/// # Safety
///
/// `surface` must be a valid engine surface.
#[inline]
pub(crate) unsafe fn surface_size(surface: *mut c_void) -> (i32, i32) {
    // SAFETY: `surface` is the engine's composite DSurface; its vtable starts
    // with one MSVC destructor slot, making GetWidth/GetHeight slots 31/32 as
    // declared in YRpp's Surface.h.
    unsafe {
        let vtable = *(surface as *const *const usize);
        let get_width: GetWidthFn = core::mem::transmute(*vtable.add(31));
        let get_height: GetHeightFn = core::mem::transmute(*vtable.add(32));
        (get_width(surface), get_height(surface))
    }
}
