
/// Universal code writer.
#[derive(Default)]
pub struct Writer {
    buffer: Vec<u8>,
}

macro_rules! define_emits {
    (#[$doc_emit:meta] $emit_fn:ident, #[$doc_emit_at:meta] $emit_at_fn:ident, $ty:ty) => {
        #[$doc_emit]
        pub fn $emit_fn(&mut self, value: $ty) {
            self.emit(&value.to_le_bytes());
        }

        #[$doc_emit_at]
        pub fn $emit_at_fn(&mut self, offset: usize, value: $ty) {
            self.emit_at(offset, &value.to_le_bytes());
        }
    };
}

/// Generic implementation of a code writer.
#[allow(dead_code)]
impl Writer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Emit a sequence of optional bytes at the end of the buffer.
    fn emit_maybe(&mut self, bytes: &[Option<u8>]) {
        self.buffer.extend(
            bytes.iter().flatten()
        );
    }

    /// Emit a byte sequence at the end of the buffer.
    fn emit(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Emit a byte sequence at the specified position.
    /// 
    /// # Panics
    /// 
    /// Panics if `[offset..offset + bytes.len()]` is outside of the self.buffer bounds.
    fn emit_at(&mut self, offset: usize, bytes: &[u8]) {
        self.buffer
            .splice(offset..offset + bytes.len(), bytes.iter().cloned());
    }

    define_emits!(
        /// Emit a byte at the end of the buffer.
        emit8, 
        /// Emit a byte at a specific offset.
        emit8_at, u8);
    define_emits!(
        /// Emit a 16-bit word at the end of the buffer.
        emit16,
        /// Emit a 16-bit word at a specific offset.
        emit16_at, u16);
    define_emits!(
        /// Emit a 32-bit dword at the end of the buffer.
        emit32,
        /// Emit a 32-bit dword at a specific offset.
        emit32_at, u32);
    define_emits!(
        /// Emit a 64-bit qword at the end of the buffer.
        emit64,
        /// Emit a 64-bit qword at a specific offset.
        emit64_at, u64);
}
