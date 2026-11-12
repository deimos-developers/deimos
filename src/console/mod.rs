//! Developer console: state, command dispatch, input and rendering.
//!
//! The pure modules (`input`, `commands`) compile on every host so their unit
//! tests can run on Linux. The engine-facing modules are Windows-only.

mod commands;
mod input;

#[cfg(windows)]
mod draw;
#[cfg(windows)]
mod hooks;
#[cfg(windows)]
mod keyboard;

#[cfg(windows)]
use std::sync::{Mutex, MutexGuard};

#[cfg(windows)]
use input::InputBuffer;

/// Mutable console state, guarded because the hook procs and the keyboard
/// poller may be called from different engine paths.
#[cfg(windows)]
pub(crate) struct ConsoleState {
    /// Whether the input box is visible and owns the keyboard.
    pub(crate) open: bool,
    /// Set when the console closes so the same frame's `KeyboardProcess` call
    /// cannot leak the closing key into the game.
    pub(crate) suppress_next: bool,
    /// Current text input.
    pub(crate) input: InputBuffer,
}

#[cfg(windows)]
impl ConsoleState {
    const fn new() -> Self {
        Self {
            open: false,
            suppress_next: false,
            input: InputBuffer::new(),
        }
    }
}

#[cfg(windows)]
static CONSOLE: Mutex<ConsoleState> = Mutex::new(ConsoleState::new());

/// Locks the console state, recovering from a poisoned mutex because a panic
/// in a hook must not permanently wedge the game.
#[cfg(windows)]
pub(crate) fn lock_state() -> MutexGuard<'static, ConsoleState> {
    CONSOLE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Returns whether the console is open.
#[cfg(windows)]
pub(crate) fn is_open() -> bool {
    lock_state().open
}

#[cfg(windows)]
fn open_console() {
    {
        let mut state = lock_state();
        state.open = true;
        state.suppress_next = false;
        state.input.clear();
    }
    // A key held down before the console opened must not count as a fresh
    // key press on the next frame.
    keyboard::synchronize_edges();
}

/// Closes the console and swallows the key that closed it.
#[cfg(windows)]
pub(crate) fn close_console() {
    let mut state = lock_state();
    state.open = false;
    state.suppress_next = true;
}

/// Toggles the console.
#[cfg(windows)]
pub(crate) fn toggle() {
    if is_open() {
        close_console();
    } else {
        open_console();
    }
}

/// Appends one character to the input buffer.
#[cfg(windows)]
pub(crate) fn push_char(character: u8) {
    lock_state().input.push(character);
}

/// Deletes the last input character.
#[cfg(windows)]
pub(crate) fn backspace() {
    lock_state().input.pop();
}

/// Executes the current input and closes the console.
#[cfg(windows)]
pub(crate) fn execute_command() {
    let line = {
        let state = lock_state();
        state.input.as_str().to_owned()
    };
    match commands::execute(&line) {
        commands::CommandResult::GiveMoney(amount) => {
            #[cfg(windows)]
            // SAFETY: the console is only reachable through the in-game draw
            // hook, after the engine has initialized CurrentPlayer.
            unsafe {
                crate::engine::house::give_money(amount);
            }
            #[cfg(not(windows))]
            let _ = amount;
        }
        commands::CommandResult::Handled
        | commands::CommandResult::Unknown
        | commands::CommandResult::Invalid => {}
    }
    close_console();
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn state_starts_closed() {
        assert!(!is_open());
    }
}
