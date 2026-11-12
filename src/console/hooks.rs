//! Hook procs and hook declarations for the developer console.

use crate::engine::hooks::{DRAW_TEXT, DRAW_TEXT_SIZE, KEYBOARD_PROCESS, KEYBOARD_PROCESS_SIZE};
use crate::syringe_api::Registers;

use super::{draw, keyboard};

/// `GScreenClass::DrawText` hook: polls input and renders the console.
#[unsafe(no_mangle)]
pub extern "C" fn Deimos_DevConsole_GScreenClass_DrawText(_registers: *mut Registers) -> u32 {
    keyboard::poll_keyboard();
    draw::draw_console();
    0
}

/// `Game::KeyboardProcess` hook: replaces the input DWORD with zero while the
/// console consumes the keyboard. Returning zero lets the engine function run
/// with neutralized input, so its own bookkeeping remains consistent.
#[unsafe(no_mangle)]
pub extern "C" fn Deimos_DevConsole_KeyboardProcess(registers: *mut Registers) -> u32 {
    if registers.is_null() {
        return 0;
    }

    let suppress = {
        let mut state = super::lock_state();
        let suppress = state.open || state.suppress_next;
        state.suppress_next = false;
        suppress
    };

    if suppress {
        // KeyboardProcess is __fastcall(DWORD&); the reference is in ECX.
        let input = unsafe { (*registers).ecx() } as *mut u32;
        if !input.is_null() {
            // SAFETY: the engine passes a writable DWORD reference in ECX.
            unsafe {
                *input = 0;
            }
        }
    }
    0
}

crate::declhook!(
    DRAW_TEXT,
    Deimos_DevConsole_GScreenClass_DrawText,
    DRAW_TEXT_SIZE
);
crate::declhook!(
    KEYBOARD_PROCESS,
    Deimos_DevConsole_KeyboardProcess,
    KEYBOARD_PROCESS_SIZE
);
