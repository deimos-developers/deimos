//! Rendering for the developer console.

use super::input::MAX_INPUT;
use super::lock_state;
use crate::engine::drawing::{
    COLOR_BLACK, COLOR_WHITE, Point2D, RectangleStruct, TEXT_NO_SHADOW, composite_surface,
    fancy_text_print_wide, fill_rect, surface_size,
};

const BOX_X: i32 = 10;
const BOX_Y: i32 = 10;
const BOX_WIDTH: i32 = 320;
const BOX_HEIGHT: i32 = 28;
const BORDER: i32 = 2;
const TEXT_OFFSET: i32 = 6;
const COLOR_BORDER: u32 = 0x8410;

/// Draws the input box and the current command line.
pub(super) fn draw_console() {
    // Copy the small, `Copy` state out of the mutex before calling into the
    // engine; the lock must not be held while the engine draws.
    let (open, input) = {
        let state = lock_state();
        (state.open, state.input)
    };
    if !open {
        return;
    }

    // SAFETY: all pointers are read from engine globals and passed straight
    // back to engine drawing functions. The engine is initialized because
    // this is called from GScreenClass::DrawText.
    unsafe {
        let surface = composite_surface();
        if surface.is_null() {
            return;
        }
        let (width, height) = surface_size(surface);
        if width <= 0 || height <= 0 {
            return;
        }

        let border = RectangleStruct {
            x: BOX_X - BORDER,
            y: BOX_Y - BORDER,
            width: BOX_WIDTH + BORDER * 2,
            height: BOX_HEIGHT + BORDER * 2,
        };
        fill_rect(surface, &border, COLOR_BORDER);

        let box_rect = RectangleStruct {
            x: BOX_X,
            y: BOX_Y,
            width: BOX_WIDTH,
            height: BOX_HEIGHT,
        };
        fill_rect(surface, &box_rect, COLOR_BLACK);

        // `write_prompt` guarantees a terminating NUL even when the input is
        // exactly MAX_INPUT long.
        let mut text = [0u16; MAX_INPUT + 4];
        let _ = input.write_prompt(&mut text);

        let bounds = RectangleStruct {
            x: 0,
            y: 0,
            width,
            height,
        };
        let location = Point2D {
            x: BOX_X + TEXT_OFFSET,
            y: BOX_Y + TEXT_OFFSET,
        };
        // The engine uses this argument as an output buffer, so keep it on
        // the writable stack.
        let mut ret_buffer = Point2D { x: 0, y: 0 };
        fancy_text_print_wide(
            &mut ret_buffer,
            text.as_ptr(),
            surface,
            &bounds,
            &location,
            COLOR_WHITE,
            0,
            TEXT_NO_SHADOW,
        );
    }
}
