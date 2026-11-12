//! Keyboard polling for the developer console.
//!
//! The engine does not expose a simple "key was pressed" queue for arbitrary
//! console keys, so this module polls `GetAsyncKeyState` once per rendered
//! frame and edge-detects key transitions. `synchronize_edges` is used when
//! the console opens so a key already held down is not treated as a fresh
//! press.

use core::sync::atomic::{AtomicU8, Ordering};

const VK_BACK: i32 = 0x08;
const VK_RETURN: i32 = 0x0D;
const VK_ESCAPE: i32 = 0x1B;
const VK_OEM_3: i32 = 0xC0;
const VK_0: i32 = 0x30;
const VK_9: i32 = 0x39;
const VK_A: i32 = 0x41;
const VK_Z: i32 = 0x5A;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(vkey: i32) -> i16;
}

static PREVIOUS_KEYS: [AtomicU8; 256] = [const { AtomicU8::new(0) }; 256];

fn is_down(vk: i32) -> bool {
    // SAFETY: GetAsyncKeyState is a side-effect-free Win32 query.
    (unsafe { GetAsyncKeyState(vk) } as u16 & 0x8000) != 0
}

fn just_pressed(vk: i32) -> bool {
    let down = is_down(vk);
    let previous = PREVIOUS_KEYS[vk as usize].swap(u8::from(down), Ordering::Relaxed);
    down && previous == 0
}

/// Re-reads every key after the console opens.
pub(super) fn synchronize_edges() {
    for vk in 0..=255 {
        let down = is_down(vk);
        PREVIOUS_KEYS[vk as usize].store(u8::from(down), Ordering::Relaxed);
    }
}

/// Polls the keyboard once per rendered frame.
pub(super) fn poll_keyboard() {
    if just_pressed(VK_OEM_3) {
        super::toggle();
        return;
    }
    if !super::is_open() {
        return;
    }

    if just_pressed(VK_RETURN) {
        super::execute_command();
        return;
    }
    if just_pressed(VK_ESCAPE) {
        super::close_console();
        return;
    }
    if just_pressed(VK_BACK) {
        super::backspace();
    }

    for vk in VK_0..=VK_9 {
        if just_pressed(vk) {
            super::push_char(b'0' + (vk - VK_0) as u8);
        }
    }
    for vk in VK_A..=VK_Z {
        if just_pressed(vk) {
            super::push_char(b'a' + (vk - VK_A) as u8);
        }
    }
}
