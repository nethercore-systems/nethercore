//! Dense table for resolving game texture handles to graphics texture handles.
//!
//! This avoids `HashMap` lookups in the inner render loop by using O(1) indexing.

use super::TextureHandle;

/// Dense mapping from game texture handle (`u32`) to `TextureHandle`.
///
/// Game texture handles are allocated sequentially starting at 1, so we can use a `Vec`
/// indexed by handle value. Handle 0 is reserved for "invalid".
#[derive(Debug)]
pub struct TextureHandleTable {
    handles: Vec<TextureHandle>,
    white: TextureHandle,
    font: TextureHandle,
}

impl TextureHandleTable {
    pub fn new() -> Self {
        Self {
            handles: vec![TextureHandle::INVALID],
            white: TextureHandle::INVALID,
            font: TextureHandle::INVALID,
        }
    }

    pub fn insert(&mut self, game_handle: u32, texture: TextureHandle) {
        match game_handle {
            u32::MAX => {
                self.white = texture;
            }
            x if x == u32::MAX - 1 => {
                self.font = texture;
            }
            _ => {
                let idx = game_handle as usize;
                if idx >= self.handles.len() {
                    self.handles.resize(idx + 1, TextureHandle::INVALID);
                }
                self.handles[idx] = texture;
            }
        }
    }

    #[inline]
    pub fn resolve(&self, game_handle: u32) -> TextureHandle {
        match game_handle {
            u32::MAX => self.white,
            x if x == u32::MAX - 1 => self.font,
            _ => self
                .handles
                .get(game_handle as usize)
                .copied()
                .unwrap_or(TextureHandle::INVALID),
        }
    }

    #[inline]
    pub fn resolve4(&self, textures: &[u32; 4]) -> [TextureHandle; 4] {
        [
            self.resolve(textures[0]),
            self.resolve(textures[1]),
            self.resolve(textures[2]),
            self.resolve(textures[3]),
        ]
    }
}

impl Default for TextureHandleTable {
    fn default() -> Self {
        Self::new()
    }
}
