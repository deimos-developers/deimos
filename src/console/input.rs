//! Pure text-input buffer used by the developer console.
//!
//! Kept independent of the engine so it can be tested on any host.

/// Maximum number of characters accepted by the console.
pub(crate) const MAX_INPUT: usize = 24;

/// Fixed-capacity ASCII input buffer.
#[derive(Clone, Copy)]
pub(crate) struct InputBuffer {
    bytes: [u8; MAX_INPUT],
    len: usize,
}

impl InputBuffer {
    /// Creates an empty buffer.
    pub(crate) const fn new() -> Self {
        Self {
            bytes: [0; MAX_INPUT],
            len: 0,
        }
    }

    /// Removes all characters.
    pub(crate) fn clear(&mut self) {
        self.bytes.fill(0);
        self.len = 0;
    }

    /// Appends a printable ASCII character, ignoring input when full.
    pub(crate) fn push(&mut self, character: u8) -> bool {
        if self.len >= MAX_INPUT {
            return false;
        }
        self.bytes[self.len] = character;
        self.len += 1;
        true
    }

    /// Removes the last character, if any.
    pub(crate) fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let character = self.bytes[self.len];
        self.bytes[self.len] = 0;
        Some(character)
    }

    /// Returns the current input as a byte slice.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    /// Returns the current input as a UTF-8 string slice.
    pub(crate) fn as_str(&self) -> &str {
        // The only writers are the keyboard layout mappings, which only add
        // ASCII letters and digits.
        core::str::from_utf8(self.as_bytes()).unwrap_or("")
    }

    /// Returns the number of characters currently stored.
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Writes the console prompt (`"> " + input + "_"`) and a terminating NUL
    /// into `out`. Returns `false` when `out` is too small.
    pub(crate) fn write_prompt(&self, out: &mut [u16]) -> bool {
        let needed = self.len() + 4;
        if out.len() < needed {
            return false;
        }

        out[0] = b'>' as u16;
        out[1] = b' ' as u16;
        for (index, byte) in self.as_bytes().iter().enumerate() {
            out[2 + index] = *byte as u16;
        }
        out[2 + self.len] = b'_' as u16;
        out[2 + self.len + 1] = 0;
        true
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_and_clear() {
        let mut buffer = InputBuffer::new();
        assert!(buffer.push(b'c'));
        assert!(buffer.push(b'a'));
        assert_eq!(buffer.as_str(), "ca");
        assert_eq!(buffer.pop(), Some(b'a'));
        assert_eq!(buffer.as_str(), "c");
        buffer.clear();
        assert_eq!(buffer.as_str(), "");
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn capacity_is_enforced() {
        let mut buffer = InputBuffer::new();
        for _ in 0..MAX_INPUT {
            assert!(buffer.push(b'x'));
        }
        assert!(!buffer.push(b'y'));
        assert_eq!(buffer.len(), MAX_INPUT);
        assert_eq!(buffer.as_str().len(), MAX_INPUT);
    }

    #[test]
    fn prompt_is_nul_terminated_at_capacity() {
        let mut input = InputBuffer::new();
        for _ in 0..MAX_INPUT {
            assert!(input.push(b'x'));
        }

        let mut prompt = [0u16; MAX_INPUT + 4];
        assert!(input.write_prompt(&mut prompt));
        assert_eq!(prompt[0], b'>' as u16);
        assert_eq!(prompt[1], b' ' as u16);
        assert_eq!(prompt[2 + MAX_INPUT], b'_' as u16);
        // The cursor must still leave room for the terminating NUL.
        assert_eq!(prompt[2 + MAX_INPUT + 1], 0);
    }

    #[test]
    fn prompt_rejects_too_small_buffer() {
        let mut input = InputBuffer::new();
        for _ in 0..MAX_INPUT {
            assert!(input.push(b'x'));
        }

        // One slot less than the prompt needs must be rejected, not truncated.
        let mut too_small = [0u16; MAX_INPUT + 3];
        assert!(!input.write_prompt(&mut too_small));
    }
}
