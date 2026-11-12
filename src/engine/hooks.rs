//! Engine hook sites shared by Deimos features.

/// `GScreenClass::DrawText` instruction patched by Phobos; the game runs this
/// every rendered frame while a mission is active.
pub(crate) const DRAW_TEXT: u32 = 0x004F_4583;
/// Size of the patched instruction sequence at [`DRAW_TEXT`].
pub(crate) const DRAW_TEXT_SIZE: u32 = 6;

/// `Game::KeyboardProcess` (`__fastcall`, input in ECX).
#[cfg(feature = "dev-console")]
pub(crate) const KEYBOARD_PROCESS: u32 = 0x0055_DEE0;
/// Size of the patched sequence at [`KEYBOARD_PROCESS`].
#[cfg(feature = "dev-console")]
pub(crate) const KEYBOARD_PROCESS_SIZE: u32 = 5;
